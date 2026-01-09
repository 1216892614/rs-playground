use bevy::prelude::*;
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(PostStartup, init_window)
        .init_resource::<MenuState>()
        .init_resource::<CameraControllerState>()
        .add_systems(Update, (
            toggle_menu,
            camera_rotation,
            camera_follow,
            player_movement,
            player_rotation,
            update_crosshair,
            player_shoot,
            bullet_movement,
            enemy_ai,
            check_collisions,
        ))
        .run();
}

// 组件
#[derive(Component)]
struct Player {
    speed: f32,
    rotation_speed: f32, // 转向速度（弧度/秒）
}

#[derive(Component)]
struct Bullet {
    lifetime: Timer,
    speed: f32,
    direction: Vec3,
}

#[derive(Component)]
struct Enemy {
    health: f32,
    speed: f32,
}

#[derive(Component)]
struct Health {
    value: f32,
}

#[derive(Component)]
struct Trajectory {
    // 弹道组件，用于管理标线和子弹发射
}

#[derive(Component)]
struct Crosshair {
    length: f32,
}

#[derive(Component)]
struct CameraController {
    // 相机相对于玩家的偏移（越肩视角）
    offset: Vec3,
    // 水平旋转角度（Y轴）
    yaw: f32,
    // 垂直旋转角度（X轴，俯仰角）
    pitch: f32,
    // 鼠标灵敏度
    sensitivity: f32,
}

// 菜单状态资源
#[derive(Resource, Default)]
struct MenuState {
    is_open: bool,
}

// 相机控制器资源（存储旋转状态）
#[derive(Resource, Default)]
struct CameraControllerState {
    yaw: f32,
    pitch: f32,
}

// 初始化场景
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 光照
    commands.spawn((
        DirectionalLight {
            illuminance: 3000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -PI / 4.0, -PI / 4.0, 0.0)),
    ));

    // 环境光
    commands.spawn(AmbientLight {
        color: Color::WHITE,
        brightness: 0.3,
        affects_lightmapped_meshes: false,
    });

    // 地面
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // 创建弹道中心标线材质（无光照）
    let crosshair_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0), // 红色标线
        unlit: true, // 不受光照影响
        ..default()
    });

    // 玩家（一个立方体代表）
    let player_entity = commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 2.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.6, 0.9))),
        Transform::from_xyz(0.0, 1.0, 0.0),
        Player { 
            speed: 5.0,
            rotation_speed: 5.0, // 转向速度
        },
        Health { value: 100.0 },
    )).id();

    // 创建弹道组件作为玩家的子实体
    let trajectory_entity = commands.spawn((
        Transform::from_xyz(0.0, 0.5, 0.0), // 相对于玩家的位置（玩家中心上方0.5单位）
        Trajectory {},
        ChildOf(player_entity),
    )).id();

    // 创建弹道中心标线作为弹道的子实体
    commands.entity(trajectory_entity).with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.02, 10.0))), // 细长的圆柱体作为标线
            MeshMaterial3d(crosshair_material),
            // 标线从弹道关节处开始，沿着NEG_Z方向延伸
            // 圆柱体中心在 (0, 0, -5.0)，这样标线从 (0,0,0) 开始向前延伸10个单位
            Transform::from_translation(Vec3::new(0.0, 0.0, -5.0)),
            Crosshair { length: 10.0 },
        ));
    });

    // 越肩视角相机
    commands.spawn((
        Camera3d::default(),
        CameraController {
            offset: Vec3::new(0.0, 1.5, 3.0), // 玩家后方上方
            yaw: 0.0,
            pitch: -0.3, // 稍微向下看
            sensitivity: 0.002,
        },
        Transform::from_xyz(0.0, 2.5, 3.0)
            .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));


    // 生成一些敌人
    for i in 0..5 {
        let angle = (i as f32) * 2.0 * PI / 5.0;
        let radius = 10.0;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 2.0, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.9, 0.2, 0.2))),
            Transform::from_xyz(x, 1.0, z),
            Enemy {
                health: 50.0,
                speed: 2.0,
            },
            Health { value: 50.0 },
        ));
    }
}

// ESC 菜单切换
fn toggle_menu(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut menu_state: ResMut<MenuState>,
    mut windows: Query<(&mut Window, &mut bevy::window::CursorOptions)>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        menu_state.is_open = !menu_state.is_open;
        
        if let Ok((mut window, mut cursor_options)) = windows.single_mut() {
            if menu_state.is_open {
                // 菜单打开：显示并解锁鼠标，将鼠标居中
                cursor_options.grab_mode = bevy::window::CursorGrabMode::None;
                cursor_options.visible = true;
                
                // 将鼠标居中到窗口中心
                let center_x = window.width() / 2.0;
                let center_y = window.height() / 2.0;
                window.set_cursor_position(Some(Vec2::new(center_x, center_y)));
            } else {
                // 菜单关闭：隐藏并锁定鼠标
                cursor_options.grab_mode = bevy::window::CursorGrabMode::Locked;
                cursor_options.visible = false;
            }
        }
    }
}

// 初始化窗口设置
fn init_window(mut windows: Query<&mut bevy::window::CursorOptions>) {
    if let Ok(mut cursor_options) = windows.single_mut() {
        // 锁定鼠标并隐藏光标
        cursor_options.grab_mode = bevy::window::CursorGrabMode::Locked;
        cursor_options.visible = false;
    }
}

// 相机旋转控制（鼠标视角）
fn camera_rotation(
    mut camera_query: Query<&mut CameraController, With<Camera3d>>,
    mut camera_state: ResMut<CameraControllerState>,
    mut mouse_motion: MessageReader<bevy::input::mouse::MouseMotion>,
    menu_state: Res<MenuState>,
) {
    if menu_state.is_open {
        return;
    }

    let mut delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        delta += event.delta;
    }

    if let Ok(mut camera) = camera_query.single_mut() {
        // 更新旋转角度
        camera_state.yaw -= delta.x * camera.sensitivity;
        camera_state.pitch -= delta.y * camera.sensitivity;
        
        // 限制俯仰角范围（避免翻转）
        camera_state.pitch = camera_state.pitch.clamp(-PI / 2.5, PI / 3.0);
        
        camera.yaw = camera_state.yaw;
        camera.pitch = camera_state.pitch;
    }
}

// 相机跟随玩家（越肩视角）
fn camera_follow(
    player_query: Query<&Transform, (With<Player>, Without<Camera3d>)>,
    mut camera_query: Query<(&mut Transform, &CameraController), With<Camera3d>>,
    menu_state: Res<MenuState>,
) {
    if menu_state.is_open {
        return;
    }

    if let Ok(player_transform) = player_query.single() {
        if let Ok((mut camera_transform, camera)) = camera_query.single_mut() {
            // 计算相机的旋转
            let yaw_quat = Quat::from_rotation_y(camera.yaw);
            let pitch_quat = Quat::from_rotation_x(camera.pitch);
            let rotation = yaw_quat * pitch_quat;
            
            // 计算相机相对于玩家的偏移（考虑旋转）
            let rotated_offset = rotation * camera.offset;
            
            // 设置相机位置
            camera_transform.translation = player_transform.translation + rotated_offset;
            
            // 计算相机看向的点（玩家前方，稍微向上）
            let look_at = player_transform.translation + Vec3::Y * 1.5;
            camera_transform.look_at(look_at, Vec3::Y);
        }
    }
}

// 玩家移动 - 基于相机方向
fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Transform, &Player), With<Player>>,
    camera_query: Query<&CameraController, With<Camera3d>>,
    time: Res<Time>,
    menu_state: Res<MenuState>,
) {
    // 如果菜单打开，不处理移动
    if menu_state.is_open {
        return;
    }
    
    if let Ok((mut transform, player)) = player_query.single_mut() {
        if let Ok(camera) = camera_query.single() {
            // 基于相机的水平旋转计算移动方向
            let yaw_quat = Quat::from_rotation_y(camera.yaw);
            
            // 相机的向前方向（只考虑水平旋转）
            let forward = yaw_quat * Vec3::NEG_Z;
            let right = yaw_quat * Vec3::X;
            
            let mut movement = Vec3::ZERO;
            
            if keyboard_input.pressed(KeyCode::KeyW) {
                movement += forward;
            }
            if keyboard_input.pressed(KeyCode::KeyS) {
                movement -= forward;
            }
            if keyboard_input.pressed(KeyCode::KeyA) {
                movement -= right;
            }
            if keyboard_input.pressed(KeyCode::KeyD) {
                movement += right;
            }
            
            movement.y = 0.0;
            movement = movement.normalize_or_zero();
            
            transform.translation += movement * player.speed * time.delta_secs();
        }
    }
}

// 玩家旋转 - 平滑转向视角方向
fn player_rotation(
    mut player_query: Query<(&mut Transform, &Player), With<Player>>,
    camera_query: Query<&CameraController, With<Camera3d>>,
    time: Res<Time>,
    menu_state: Res<MenuState>,
) {
    if menu_state.is_open {
        return;
    }

    if let Ok((mut transform, player)) = player_query.single_mut() {
        if let Ok(camera) = camera_query.single() {
            // 玩家直接使用相机的 yaw 角度作为目标旋转
            // 相机的 yaw 已经是绕 Y 轴的旋转角度
            let target_rotation = Quat::from_rotation_y(camera.yaw);
            
            // 当前旋转
            let current_rotation = transform.rotation;
            
            // 使用 slerp 平滑插值旋转
            // 计算旋转速度因子（基于时间步长和转向速度）
            let rotation_factor = (player.rotation_speed * time.delta_secs()).min(1.0);
            
            // 平滑旋转到目标方向
            transform.rotation = current_rotation.slerp(target_rotation, rotation_factor);
        }
    }
}

// 更新弹道和标线方向 - 水平旋转跟随玩家（作为子实体自动跟随），俯仰角跟随相机
fn update_crosshair(
    mut trajectory_query: Query<&mut Transform, (With<Trajectory>, Without<Player>)>,
    mut crosshair_query: Query<&mut Transform, (With<Crosshair>, Without<Trajectory>)>,
    camera_query: Query<&CameraController, (With<Camera3d>, Without<Trajectory>)>,
    menu_state: Res<MenuState>,
) {
    // 如果菜单打开，隐藏标线
    if menu_state.is_open {
        if let Ok(mut crosshair_transform) = crosshair_query.single_mut() {
            crosshair_transform.scale = Vec3::ZERO;
        }
        return;
    }
    
    if let Ok(mut trajectory_transform) = trajectory_query.single_mut() {
        if let Ok(camera) = camera_query.single() {
            // 弹道作为玩家的子实体，水平旋转会自动跟随玩家
            // 只需要设置俯仰角（X轴旋转）跟随相机
            
            // 相机的俯仰角（X轴）
            let camera_pitch = camera.pitch;
            
            // 只设置俯仰角旋转（X轴）
            let pitch_quat = Quat::from_rotation_x(camera_pitch);
            trajectory_transform.rotation = pitch_quat;
            
            // 更新标线的旋转（相对于弹道）
            if let Ok(mut crosshair_transform) = crosshair_query.single_mut() {
                // 圆柱体默认沿Y轴（垂直），需要旋转90度让它沿Z轴（水平，向前）
                // 标线沿着弹道的NEG_Z方向延伸
                let base_rotation = Quat::from_rotation_x(PI / 2.0); // 让圆柱体水平
                crosshair_transform.rotation = base_rotation;
                crosshair_transform.scale = Vec3::new(1.0, 1.0, 1.0);
            }
        }
    }
}

// 玩家射击 - 从弹道位置发射子弹
fn player_shoot(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    trajectory_query: Query<&GlobalTransform, (With<Trajectory>, Without<Player>)>,
    menu_state: Res<MenuState>,
) {
    // 如果菜单打开，不处理射击
    if menu_state.is_open {
        return;
    }
    
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Ok(trajectory_global_transform) = trajectory_query.single() {
            // 使用 GlobalTransform 获取弹道的世界位置和旋转
            // 弹道的旋转已经包含了玩家的水平旋转和相机的俯仰角（通过父子关系）
            
            // 计算瞄准方向（向前是NEG_Z）
            let aim_direction = trajectory_global_transform.rotation() * Vec3::NEG_Z;
            
            // 获取弹道的世界位置
            let spawn_pos = trajectory_global_transform.translation();
            let direction = aim_direction.normalize();
            
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.1))),
                MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 0.0))),
                Transform::from_translation(spawn_pos),
                Bullet {
                    lifetime: Timer::from_seconds(3.0, TimerMode::Once),
                    speed: 20.0,
                    direction: direction,
                },
            ));
        }
    }
}

// 子弹移动
fn bullet_movement(
    mut commands: Commands,
    mut bullet_query: Query<(Entity, &mut Transform, &mut Bullet)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut bullet) in bullet_query.iter_mut() {
        bullet.lifetime.tick(time.delta());
        
        if bullet.lifetime.is_finished() {
            commands.entity(entity).despawn();
        } else {
            transform.translation += bullet.direction * bullet.speed * time.delta_secs();
        }
    }
}

// 敌人 AI
fn enemy_ai(
    mut enemy_query: Query<(&mut Transform, &Enemy), (With<Enemy>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (mut transform, enemy) in enemy_query.iter_mut() {
            let direction = (player_transform.translation - transform.translation)
                .normalize_or_zero();
            
            transform.translation += direction * enemy.speed * time.delta_secs();
            
            // 面向玩家
            if direction.length_squared() > 0.0 {
                let look_direction = direction.normalize();
                transform.rotation = Quat::from_rotation_y(
                    look_direction.z.atan2(look_direction.x) - PI / 2.0
                );
            }
        }
    }
}

// 碰撞检测
fn check_collisions(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform), With<Bullet>>,
    mut enemy_query: Query<(Entity, &mut Health, &Transform), With<Enemy>>,
) {
    for (bullet_entity, bullet_transform) in bullet_query.iter() {
        for (enemy_entity, mut health, enemy_transform) in enemy_query.iter_mut() {
            let distance = bullet_transform.translation.distance(enemy_transform.translation);
            
            if distance < 1.5 {
                // 击中敌人
                health.value -= 25.0;
                commands.entity(bullet_entity).despawn();
                
                if health.value <= 0.0 {
                    commands.entity(enemy_entity).despawn();
                }
            }
        }
    }
}
