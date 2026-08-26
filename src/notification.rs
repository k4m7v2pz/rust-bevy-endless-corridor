//! 通知系统
//!
//! 迁移自 python_arcade `src/engine/ui/notification_system.py`。
//! 在屏幕左上角显示游戏动作通知（发现线索 / 触发陷阱 / 获得物品 等），
//! 与 HUD 互补，提供更丰富的 UI 反馈。
//!
//! 设计：
//! - `NotificationQueue` Resource 按优先级维护活跃通知
//! - 每条通知有 duration，超时自动移除
//! - 通过 Bevy Event（ClueDiscoveredEvent / TrapTriggeredEvent）自动响应
//! - UI 走 NodeBundle + TextBundle，仿 game_ui::spawn_hud 模式

use bevy::prelude::*;
use std::collections::VecDeque;

use crate::trap::{ClueDiscoveredEvent, TrapTriggeredEvent};

// ---------- 通知数据 ----------

/// 通知类型（决定颜色与图标）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    /// 发现线索（浅蓝）
    ClueFound,
    /// 获得物品（浅绿）
    ItemObtained,
    /// 触发陷阱（橙红警告）
    TrapTriggered,
    /// 行动结果（浅黄）
    ActionResult,
    /// 系统消息（灰）
    SystemMessage,
    /// 成功（绿）
    Success,
}

impl NotificationType {
    fn color(self) -> Color {
        match self {
            NotificationType::ClueFound => Color::srgb(0.55, 0.85, 1.0),
            NotificationType::ItemObtained => Color::srgb(0.6, 1.0, 0.6),
            NotificationType::TrapTriggered => Color::srgb(1.0, 0.65, 0.35),
            NotificationType::ActionResult => Color::srgb(1.0, 1.0, 0.55),
            NotificationType::SystemMessage => Color::srgb(0.8, 0.8, 0.8),
            NotificationType::Success => Color::srgb(0.6, 1.0, 0.6),
        }
    }

    fn icon(self) -> &'static str {
        match self {
            NotificationType::ClueFound => "🔍",
            NotificationType::ItemObtained => "📦",
            NotificationType::TrapTriggered => "⚠️",
            NotificationType::ActionResult => "→",
            NotificationType::SystemMessage => "•",
            NotificationType::Success => "✓",
        }
    }
}

/// 单条通知
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
    /// 剩余显示时间（秒），降到 0 即移除
    pub remaining: f32,
    /// 优先级（数字越大越优先，排序在前）
    pub priority: i32,
}

// ---------- 资源: 通知队列 ----------

/// 通知队列资源
///
/// 对应 Python `NotificationSystem`。
/// `max_notifications` 限制同时显示数，超出时移除优先级最低的。
#[derive(Resource, Debug)]
pub struct NotificationQueue {
    pub notifications: VecDeque<Notification>,
    pub max_notifications: usize,
}

impl Default for NotificationQueue {
    fn default() -> Self {
        Self {
            notifications: VecDeque::new(),
            max_notifications: 5,
        }
    }
}

impl NotificationQueue {
    /// 推入一条通知，按优先级插入合适位置
    pub fn push(&mut self, message: impl Into<String>, notification_type: NotificationType, duration: f32, priority: i32) {
        let notification = Notification {
            message: message.into(),
            notification_type,
            remaining: duration,
            priority,
        };

        // 按优先级降序插入（priority 高的在前）
        let mut inserted = false;
        for i in 0..self.notifications.len() {
            if self.notifications[i].priority < priority {
                self.notifications.insert(i, notification.clone());
                inserted = true;
                break;
            }
        }
        if !inserted {
            self.notifications.push_back(notification);
        }

        // 限制最大数量：移除末尾（优先级最低的）
        while self.notifications.len() > self.max_notifications {
            self.notifications.pop_back();
        }
    }

    /// 清空所有通知
    pub fn clear(&mut self) {
        self.notifications.clear();
    }
}

// ---------- UI 组件标记 ----------

/// 通知列表根容器（左上角，HUD 下方）
#[derive(Component)]
pub struct NotificationRoot;

/// 单条通知的 Text entity，带索引以对应队列位置
#[derive(Component)]
pub struct NotificationText(pub usize);

// ---------- 系统常量 ----------

/// 同时渲染的最大通知行数（与 NotificationQueue.max_notifications 对齐）
const MAX_VISIBLE_NOTIFICATIONS: usize = 5;
/// 通知淡出阈值（剩余时间低于此值时闪烁）
const FADE_THRESHOLD: f32 = 1.0;

// ---------- UI 初始化 ----------

/// 创建通知列表根容器（左上角，HUD 下方）
///
/// 在 HUD 启动时一起调用，或独立调用。容器固定创建 MAX_VISIBLE_NOTIFICATIONS
/// 个 Text entity，按队列内容更新文字与可见性。
pub fn setup_notification_ui(mut commands: Commands, ui_camera_query: Query<Entity, With<crate::UiCamera>>) {
    let Ok(ui_camera) = ui_camera_query.get_single() else { return };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(60.0),   // 蹲在 HUD（top=10）下方
                    left: Val::Px(12.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                visibility: Visibility::Visible,
                ..default()
            },
            TargetCamera(ui_camera),
            NotificationRoot,
        ))
        .with_children(|parent| {
            for i in 0..MAX_VISIBLE_NOTIFICATIONS {
                parent.spawn((
                    TextBundle {
                        visibility: Visibility::Hidden, // 默认隐藏，有内容才显
                        ..TextBundle::from_section(
                            "",
                            TextStyle {
                                font_size: 15.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        )
                    },
                    NotificationText(i),
                ));
            }
        });
}

// ---------- 事件响应: 自动推入通知 ----------

/// 监听线索发现事件，推入通知
pub fn notification_on_clue(
    mut events: EventReader<ClueDiscoveredEvent>,
    mut queue: ResMut<NotificationQueue>,
) {
    for ev in events.read() {
        let msg = if ev.title.is_empty() {
            format!("🔍 {}", ev.content)
        } else {
            format!("🔍 {} — {}", ev.title, ev.content)
        };
        queue.push(msg, NotificationType::ClueFound, 4.0, 2);
    }
}

/// 监听陷阱触发事件，推入通知
pub fn notification_on_trap(
    mut events: EventReader<TrapTriggeredEvent>,
    mut queue: ResMut<NotificationQueue>,
) {
    for ev in events.read() {
        let lethal_tag = if ev.lethal { "致命" } else { "" };
        let msg = format!(
            "⚠️ 触发陷阱 {}! 理智 -{:.0} {}",
            ev.trap_id, ev.damage, lethal_tag
        );
        queue.push(msg, NotificationType::TrapTriggered, 3.5, 3);
    }
}

// ---------- 更新系统: 超时移除 + UI 同步 ----------

/// 更新通知状态：移除超时的，同步 UI 文字与可见性
pub fn notification_update_system(
    time: Res<Time>,
    mut queue: ResMut<NotificationQueue>,
    mut texts: Query<(&NotificationText, &mut Text, &mut Visibility)>,
) {
    let dt = time.delta_seconds();

    // 移除超时通知（显式循环，retain 里是 &Notification 无法改 remaining）
    let mut i = 0;
    while i < queue.notifications.len() {
        queue.notifications[i].remaining -= dt;
        if queue.notifications[i].remaining <= 0.0 {
            queue.notifications.remove(i);
        } else {
            i += 1;
        }
    }

    // 同步 UI：遍历固定数量的 Text entity，按索引对应队列
    for (idx, mut text, mut vis) in texts.iter_mut() {
        let i = idx.0;
        match queue.notifications.get(i) {
            Some(n) => {
                // 文字内容：图标 + 消息
                text.sections[0].value = format!("{} {}", n.notification_type.icon(), n.message);
                // 颜色：临到时闪烁
                let color = if n.remaining < FADE_THRESHOLD {
                    let flash = (n.remaining * 10.0).sin().abs();
                    let base = n.notification_type.color();
                    let s = base.to_srgba();
                    Color::srgba(s.red, s.green, s.blue, s.alpha * flash)
                } else {
                    n.notification_type.color()
                };
                text.sections[0].style.color = color;
                // 可见
                *vis = Visibility::Visible;
            }
            None => {
                // 无对应队列项则隐藏
                *vis = Visibility::Hidden;
            }
        }
    }
}

// ---------- 便捷推入 API（供其他系统手动调用）----------

impl NotificationQueue {
    /// 显示"获得物品"通知
    pub fn show_item_obtained(&mut self, item_name: &str, quantity: u32) {
        let msg = if quantity > 1 {
            format!("📦 获得了 {}x {}", quantity, item_name)
        } else {
            format!("📦 获得了 {}", item_name)
        };
        self.push(msg, NotificationType::ItemObtained, 3.0, 1);
    }

    /// 显示"行动结果"通知
    pub fn show_action_result(&mut self, action: &str, result: &str) {
        self.push(
            format!("→ {}: {}", action, result),
            NotificationType::ActionResult,
            2.5,
            0,
        );
    }

    /// 显示系统消息
    pub fn show_system(&mut self, message: &str) {
        self.push(
            format!("• {}", message),
            NotificationType::SystemMessage,
            3.0,
            0,
        );
    }

    /// 显示成功通知
    pub fn show_success(&mut self, message: &str) {
        self.push(
            format!("✓ {}", message),
            NotificationType::Success,
            3.0,
            1,
        );
    }
}

// ---------- Plugin ----------

pub struct NotificationPlugin;

impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NotificationQueue>()
            .add_systems(
                Update,
                (
                    notification_on_clue,
                    notification_on_trap,
                    notification_update_system,
                )
                    .chain(),
            );
    }
}

// ---------- 单元测试 ----------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_insertion_order() {
        let mut q = NotificationQueue::default();
        q.push("low", NotificationType::SystemMessage, 1.0, 0);
        q.push("high", NotificationType::ClueFound, 1.0, 5);
        q.push("mid", NotificationType::TrapTriggered, 1.0, 2);

        let msgs: Vec<_> = q.notifications.iter().map(|n| n.message.as_str()).collect();
        assert_eq!(msgs, vec!["high", "mid", "low"]);
    }

    #[test]
    fn max_notifications_trims_lowest() {
        let mut q = NotificationQueue::default();
        q.max_notifications = 2;
        q.push("a", NotificationType::SystemMessage, 1.0, 0);
        q.push("b", NotificationType::SystemMessage, 1.0, 1);
        q.push("c", NotificationType::SystemMessage, 1.0, 2);

        assert_eq!(q.notifications.len(), 2);
        // 优先级最低的 "a" 应被移除
        let msgs: Vec<_> = q.notifications.iter().map(|n| n.message.as_str()).collect();
        assert_eq!(msgs, vec!["c", "b"]);
    }

    #[test]
    fn timeout_removal() {
        let mut q = NotificationQueue::default();
        q.push("temp", NotificationType::SystemMessage, 0.5, 0);
        q.notifications[0].remaining = 0.1;
        // 模拟一帧流逝（显式循环，retain 里是 &Notification 无法改 remaining）
        let mut i = 0;
        while i < q.notifications.len() {
            q.notifications[i].remaining -= 0.2;
            if q.notifications[i].remaining <= 0.0 {
                q.notifications.remove(i);
            } else {
                i += 1;
            }
        }
        assert!(q.notifications.is_empty());
    }

    #[test]
    fn convenience_methods_format_correctly() {
        let mut q = NotificationQueue::default();
        q.show_item_obtained("钥匙", 3);
        assert!(q.notifications[0].message.contains("3x 钥匙"));
        q.show_action_result("搜索", "发现线索");
        assert!(q.notifications.iter().any(|n| n.message.contains("搜索: 发现线索")));
        q.show_success("门已解锁");
        assert!(q.notifications.iter().any(|n| n.message.starts_with("✓")));
    }
}
