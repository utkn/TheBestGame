use rand::Rng;

use crate::{
    character::CharacterInsights,
    controller::ControlCommand,
    item::EquipmentSlot,
    physics::{ColliderInsights, ProjectileInsights},
    prelude::*,
    vehicle::VehicleInsights,
};

#[derive(Clone, Debug)]
pub enum AiTaskOutput {
    QueueFront(AiTask),
    IssueCmd(ControlCommand),
}

#[derive(Clone, Debug)]
pub enum AiTask {
    Attack {
        target: EntityRef,
    },
    MoveToPos {
        x: f32,
        y: f32,
        persistent: bool,
        scale_obstacles: bool,
    },
    Routine,
    ScaleObstacle,
}

impl AiTask {
    pub fn evaluate(self, actor: &EntityRef, state: &State) -> Vec<AiTaskOutput> {
        match self {
            AiTask::Attack { target } => attack_handler(target, actor, state),
            AiTask::MoveToPos {
                x,
                y,
                persistent,
                scale_obstacles,
            } => move_to_pos_handler(x, y, persistent, scale_obstacles, actor, state),
            AiTask::Routine => routine_handler(actor, state),
            AiTask::ScaleObstacle => scale_obstacle_handler(actor, state),
        }
    }
}

fn attack_handler(target: EntityRef, actor: &EntityRef, state: &State) -> Vec<AiTaskOutput> {
    // Cancel the attack if the target is no longer valid.
    if !state.is_valid(&target) {
        return vec![
            AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(0., 0.)),
            AiTaskOutput::IssueCmd(ControlCommand::EquipmentUninteract(EquipmentSlot::LeftHand)),
        ];
    }
    let insights = StateInsights::of(state);
    // Handle the case that we cannot see the target anymore.
    let can_see = insights
        .visibles_of_character(actor)
        .map(|visibles| visibles.contains(&target))
        .unwrap_or(false);
    if !can_see {
        // Get the last seen position of the target.
        let last_seen_pos = insights
            .transform_of(&target)
            .map(|trans| (trans.x, trans.y));
        // Start an unpersitent movement to the last seen position of the target.
        if let Some(last_seen_pos) = last_seen_pos {
            return vec![
                AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(0., 0.)),
                AiTaskOutput::IssueCmd(ControlCommand::EquipmentUninteract(
                    EquipmentSlot::LeftHand,
                )),
                AiTaskOutput::QueueFront(AiTask::MoveToPos {
                    x: last_seen_pos.0,
                    y: last_seen_pos.1,
                    persistent: true,
                    scale_obstacles: true,
                }),
            ];
        } else {
            return vec![
                AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(0., 0.)),
                AiTaskOutput::IssueCmd(ControlCommand::EquipmentUninteract(
                    EquipmentSlot::LeftHand,
                )),
            ];
        }
    }
    // If we can see the target, keep attacking it by looking at it.
    let dpos = insights.pos_diff(&target, actor).unwrap_or_default();
    let dir = notan::math::vec2(dpos.0, dpos.1).normalize();
    let target_deg = dir.angle_between(notan::math::vec2(1., 0.)).to_degrees();
    return vec![
        AiTaskOutput::QueueFront(AiTask::Attack { target }),
        AiTaskOutput::IssueCmd(ControlCommand::SetTargetRotation(target_deg)),
        AiTaskOutput::IssueCmd(ControlCommand::EquipmentInteract(EquipmentSlot::LeftHand)),
    ];
}

fn enemy_on_sight(actor: &EntityRef, state: &State) -> Option<EntityRef> {
    let insights = StateInsights::of(state);
    let visibles = insights.visibles_of_character(actor).unwrap_or_default();
    let target = visibles.into_iter().find(|e| {
        insights.is_character(e)
            || (insights.is_vehicle(e)
                && insights
                    .driver_of(e)
                    .map(|driver| insights.is_character(driver))
                    .unwrap_or(false))
    });
    target
}

fn move_towards_projectile(actor: &EntityRef, state: &State) -> Option<(f32, f32)> {
    let insights = StateInsights::of(state);
    let (vx, vy) = insights
        .new_hitters_of(actor)
        .into_iter()
        .next()
        .map(|(_, hit_vel)| hit_vel)?;
    let actor_trans = insights.transform_of(actor).unwrap();
    let rev_dir = notan::math::vec2(-*vx, -*vy).normalize();
    let target_pos = notan::math::vec2(actor_trans.x, actor_trans.y) + rev_dir * 150.;
    Some((target_pos.x, target_pos.y))
}

fn get_priority_actions(actor: &EntityRef, state: &State) -> Vec<AiTaskOutput> {
    // Move towards the projectile.
    if let Some((target_x, target_y)) = move_towards_projectile(actor, state) {
        return vec![AiTaskOutput::QueueFront(AiTask::MoveToPos {
            x: target_x,
            y: target_y,
            persistent: false,
            scale_obstacles: true,
        })];
    }
    // Attack on sight.
    if let Some(target) = enemy_on_sight(actor, state) {
        return vec![AiTaskOutput::QueueFront(AiTask::Attack { target })];
    }
    return vec![];
}

fn routine_handler(actor: &EntityRef, state: &State) -> Vec<AiTaskOutput> {
    let mut priority_actions = get_priority_actions(actor, state);
    priority_actions.insert(0, AiTaskOutput::QueueFront(AiTask::Routine));
    priority_actions
}

fn get_dpos(
    target_x: &f32,
    target_y: &f32,
    actor: &EntityRef,
    state: &State,
) -> Option<(f32, f32)> {
    let insights = StateInsights::of(state);
    let actor_pos = insights.transform_of(actor)?;
    Some((target_x - actor_pos.x, target_y - actor_pos.y))
}

fn reached_destination(target_x: &f32, target_y: &f32, actor: &EntityRef, state: &State) -> bool {
    get_dpos(target_x, target_y, actor, state)
        .map(|dpos| dpos.0.abs() <= 5. && dpos.1.abs() <= 5.)
        .unwrap_or(true)
}

fn move_to_pos_handler(
    target_x: f32,
    target_y: f32,
    persistent: bool,
    scale_obstacles: bool,
    actor: &EntityRef,
    state: &State,
) -> Vec<AiTaskOutput> {
    // Reached destination.
    if reached_destination(&target_x, &target_y, actor, state) {
        return vec![AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(
            0., 0.,
        ))];
    }
    // First, get the priority actions.
    let mut urgent_actions = get_priority_actions(actor, state);
    // If we have no priority actions and we encountered an obstacle, issue obstacle scaling.
    if urgent_actions.is_empty()
        && scale_obstacles
        && StateInsights::of(state).concrete_contacts_of(actor).len() > 0
    {
        urgent_actions.push(AiTaskOutput::QueueFront(AiTask::ScaleObstacle));
        // If persistent, maintain itself.
        if persistent {
            urgent_actions.insert(
                0,
                AiTaskOutput::QueueFront(AiTask::MoveToPos {
                    x: target_x,
                    y: target_y,
                    persistent: true,
                    scale_obstacles: true,
                }),
            );
        }
    }
    if urgent_actions.len() > 0 {
        urgent_actions.push(AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(
            0., 0.,
        )));
        return urgent_actions;
    }
    // Approach to the target.
    if let Some(dpos) = get_dpos(&target_x, &target_y, actor, state) {
        let dir = notan::math::vec2(dpos.0, dpos.1).normalize();
        let target_deg = dir.angle_between(notan::math::vec2(1., 0.)).to_degrees();
        let target_vel = dir * 300.;
        return vec![
            AiTaskOutput::IssueCmd(ControlCommand::SetTargetRotation(target_deg)),
            AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(
                target_vel.x,
                target_vel.y,
            )),
            AiTaskOutput::QueueFront(AiTask::MoveToPos {
                x: target_x,
                y: target_y,
                persistent,
                scale_obstacles,
            }),
        ];
    }
    // End the task if delta pos couldn't be found.
    return vec![AiTaskOutput::IssueCmd(ControlCommand::SetTargetVelocity(
        0., 0.,
    ))];
}

fn scale_obstacle_handler(actor: &EntityRef, state: &State) -> Vec<AiTaskOutput> {
    let insights = StateInsights::of(state);
    if let Some(overlap) = insights.concrete_contact_overlaps_of(actor).first() {
        let actor_trans = insights.transform_of(actor).unwrap();
        let mut dev = rand::thread_rng().gen_range(45_f32..80_f32);
        if rand::random() {
            dev *= -1.
        }
        let side_dir = notan::math::Vec2::from_angle(dev.to_radians())
            .rotate(notan::math::vec2(-overlap.0, -overlap.1));
        let new_pos = notan::math::vec2(actor_trans.x, actor_trans.y) + side_dir * 40.;
        // Replace itself with a nonpersistent movement from the obstacle.
        return vec![AiTaskOutput::QueueFront(AiTask::MoveToPos {
            x: new_pos.x,
            y: new_pos.y,
            persistent: false,
            scale_obstacles: false,
        })];
    }
    return vec![];
}
