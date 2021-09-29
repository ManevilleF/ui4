use bevy::prelude::*;
use std::borrow::Borrow;
use ui4::{childable::tracked::TrackedItemObserver, prelude::*};

struct UiAssets {
    background: Handle<ColorMaterial>,
    button: Handle<ColorMaterial>,
    button_hover: Handle<ColorMaterial>,
    button_click: Handle<ColorMaterial>,
    text_style: TextStyle,
    transparent: Handle<ColorMaterial>,
}

fn init_system(
    mut commands: Commands,
    mut assets: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(UiAssets {
        background: assets.add(Color::BLACK.into()),
        transparent: assets.add(Color::NONE.into()),
        button: assets.add(Color::DARK_GRAY.into()),
        button_hover: assets.add(Color::GRAY.into()),
        button_click: assets.add(Color::SILVER.into()),
        text_style: TextStyle {
            color: Color::WHITE,
            font: asset_server.load("FiraMono-Medium.ttf"),
            font_size: 32.0,
        },
    })
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugin(Ui4Plugin(root))
        .add_plugin(bevy_inspector_egui::WorldInspectorPlugin::default())
        .add_startup_system(init_system);

    app.world.spawn().insert_bundle(UiCameraBundle::default());

    app.run()
}

fn root(ctx: &mut Ctx) {
    #[derive(Component)]
    struct State(i32);

    #[derive(Component)]
    struct List(TrackedVec<String>);

    let state = ctx.component();
    let list = ctx.component();
    let this = ctx.this();

    fn m<'w>(list: &'w List) -> &'w TrackedVec<String> {
        &list.0
    }

    ctx.with_bundle(NodeBundle::default())
        .with(Style {
            size: Size {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
            },
            flex_direction: FlexDirection::ColumnReverse,
            ..Default::default()
        })
        .with(res().map(|assets: &UiAssets| assets.background.clone()))
        .with(State(0))
        .children(|ctx: &mut McCtx| {
            ctx.c(text("Hello!".to_string()))
                .c(text("How are you doing?".to_string()))
                .c(button("Increment".to_string(), move |world| {
                    world.get_mut::<State>(this).unwrap().0 += 1;
                }))
                .c(button("Decrement".to_string(), move |world| {
                    world.get_mut::<State>(this).unwrap().0 -= 1;
                }))
                .c(text(
                    state.map(|s: &State| format!("The number is {}", s.0)),
                ));
        })
        .children(list.map(m).each(|label| {
            //
            |ctx: &mut McCtx| {}
        }))
        .children(
            res()
                .map(|time: &Time| time.seconds_since_startup() as usize % 2 == 0)
                .dedup()
                .map(|b: &bool| {
                    let b = *b;
                    move |ctx: &mut McCtx| {
                        if b {
                            ctx.c(text("Now you see me".to_string()));
                        }
                    }
                }),
        );
}

fn count<M>(label: impl IntoObserver<String, M>) -> impl FnOnce(&mut Ctx) {
    #[derive(Component)]
    struct State(i32);

    move |ctx: &mut Ctx| {
        let component = ctx.component();
        let entity = ctx.this();
        ctx.with_bundle(NodeBundle::default())
            .with(Style {
                align_self: AlignSelf::FlexStart,
                ..Default::default()
            })
            .with(res().map(|assets: &UiAssets| assets.transparent.clone()))
            .with(State(0))
            .children(move |ctx: &mut McCtx| {
                ctx.c(text(label))
                    .c(button("+".to_string(), move |w| {
                        w.get_mut::<State>(entity).unwrap().0 += 1;
                    }))
                    .c(|ctx| {
                        text(component.map(|x: &State| x.0.to_string()))(ctx);
                        ctx.with(Style {
                            align_self: AlignSelf::FlexStart,
                            min_size: Size {
                                width: Val::Px(50.0),
                                height: Val::Undefined,
                            },
                            max_size: Size {
                                width: Val::Undefined,
                                height: Val::Px(30.),
                            },
                            ..Default::default()
                        });
                    })
                    .c(button("-".to_string(), move |w| {
                        w.get_mut::<State>(entity).unwrap().0 -= 1;
                    }));
            });
    }
}

fn text<O: IntoObserver<String, M>, M>(text: O) -> impl FnOnce(&mut Ctx) {
    move |ctx: &mut Ctx| {
        ctx.with_bundle(TextBundle::default())
            .with(Style {
                align_self: AlignSelf::FlexStart,
                ..Default::default()
            })
            .with(res().and(text.into_observable()).map(
                move |(assets, text): (&UiAssets, O::ObserverReturn<'_, '_>)| {
                    Text::with_section(text.borrow(), assets.text_style.clone(), Default::default())
                },
            ));
    }
}

fn button<O: IntoObserver<String, M>, M: 'static>(
    t: O,
    on_click: impl Fn(&mut World) + Send + Sync + 'static,
) -> impl FnOnce(&mut Ctx) {
    move |ctx: &mut Ctx| {
        let component = ctx.component();
        ctx.with_bundle(ButtonBundle::default())
            .with(Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            })
            .with(
                res()
                    .and(component)
                    .map(
                        |(assets, interaction): (&UiAssets, &Interaction)| match interaction {
                            Interaction::Clicked => assets.button_click.clone(),
                            Interaction::Hovered => assets.button_hover.clone(),
                            Interaction::None => assets.button.clone(),
                        },
                    ),
            )
            .with(FuncScratch::default())
            .with(ClickFunc(ButtonFunc::new(on_click)))
            .child(text(t));
    }
}
