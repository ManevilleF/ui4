use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use std::sync::Arc;
use std::time::Duration;

#[derive(Component)]
pub struct TextBox(pub usize);
#[derive(Component, Clone)]
pub struct TextBoxFunc(Arc<dyn Fn(&mut World) -> &mut String + Send + Sync>);

impl TextBoxFunc {
    pub fn new(f: impl Fn(&mut World) -> &mut String + Send + Sync + 'static) -> Self {
        Self(Arc::new(f))
    }

    fn get<'w>(&self, world: &'w mut World) -> &'w mut String {
        (self.0)(world)
    }
}

#[derive(Component)]
pub struct Focused;

pub(crate) struct TextBoxSystemState {
    state: SystemState<(
        EventReader<'static, 'static, ReceivedCharacter>,
        Query<'static, 'static, (&'static TextBoxFunc, &'static mut TextBox), With<Focused>>,
        Res<'static, Input<KeyCode>>,
        Res<'static, Time>,
    )>,
    tmp_chars: Vec<char>,
    timer: Timer,
}

impl FromWorld for TextBoxSystemState {
    fn from_world(world: &mut World) -> Self {
        TextBoxSystemState {
            state: SystemState::new(world),
            tmp_chars: Vec::new(),
            timer: Timer::new(Duration::from_millis(100), true),
        }
    }
}

impl TextBoxSystemState {
    pub(crate) fn run(&mut self, world: &mut World) {
        let (mut reader, mut q, inp, time) = self.state.get_mut(world);
        self.tmp_chars.extend(reader.iter().map(|rc| rc.char));
        if let Some((tbf, cursor)) = q.get_single_mut().ok() {
            let pl = inp.pressed(KeyCode::Left);
            let pr = inp.pressed(KeyCode::Right);
            let jpl = inp.just_pressed(KeyCode::Left);
            let jpr = inp.just_pressed(KeyCode::Right);
            let mut cursor = cursor.0;
            let delta = time.delta();
            let string = tbf.clone().get(world);
            if jpl || jpr {
                self.timer.reset();
            }
            if pr || pl {
                self.timer.tick(delta);
            }
            if (pr || pl) && (jpl || jpr || self.timer.just_finished()) {
                if pl {
                    move_left(string, &mut cursor);
                } else if pr {
                    move_right(string, &mut cursor);
                }
            }
            if !self.tmp_chars.is_empty() {
                for c in self.tmp_chars.drain(..) {
                    const BACKSPACE: char = '\u{8}';
                    const DELETE: char = '\u{7f}';
                    const RETURN: char = '\r';
                    match c {
                        c if !c.is_control() => {
                            insert_char(string, &mut cursor, c);
                        }
                        BACKSPACE => {
                            move_left(string, &mut cursor);
                            remove_char(string, &mut cursor);
                        }
                        DELETE => remove_char(string, &mut cursor),
                        RETURN => {}
                        _ => {}
                    }
                }
            }
            let (_, mut q, _, _) = self.state.get_mut(world);
            q.single_mut().1 .0 = cursor;
        }
    }
}

fn insert_char(string: &mut String, index: &mut usize, c: char) {
    string.insert(*index, c);
    *index += c.len_utf8();
}

fn remove_char(string: &mut String, index: &mut usize) {
    if *index < string.len() {
        string.remove(*index);
    }
}

fn move_left(string: &mut String, index: &mut usize) {
    loop {
        if *index == 0 {
            break;
        }
        *index -= 1;
        if string.is_char_boundary(*index) {
            break;
        }
    }
}

fn move_right(string: &mut String, index: &mut usize) {
    loop {
        if *index == string.len() {
            break;
        }
        *index += 1;
        if string.is_char_boundary(*index) {
            break;
        }
    }
}

pub(crate) fn focus_textbox_system(
    mut commands: Commands,
    input: Res<Input<MouseButton>>,
    q: Query<(Entity, &Interaction, Option<&Focused>), With<TextBox>>,
) {
    if input.just_pressed(MouseButton::Left) {
        for (entity, interaction, has_focused) in q.iter() {
            match (interaction, has_focused.is_some()) {
                (Interaction::Clicked, false) => {
                    commands.entity(entity).insert(Focused);
                }
                (Interaction::None, true) => {
                    commands.entity(entity).remove::<Focused>();
                }
                _ => {}
            }
        }
    }
}
