use makara::prelude::*;
use bevy::prelude::*;

pub fn setup_styles(mut styles: ResMut<CustomStyle>) {
    styles.bind_class(
        "menu-btn",
        Style::new()
            .no_shadow()
            .border_radius(px(0))
    );

    styles.bind_id(
        "operator-panel",
        Style::new()
            .width(px(240))
            .min_width(px(240))
            .height(percent(100))
            .padding(px(5))
            .border_radius(px(5))
            .justify_content(JustifyContent::Start)
            .shadow(BoxShadow::new(
                Color::BLACK.with_alpha(0.8),
                px(0.0),
                px(1.0),
                px(1.0),
                px(2.0),
            ))
            .background_color("white")
    );

    styles.bind_id(
        "property-panel",
        Style::new()
            .width(px(240))
            .min_width(px(240))
            .height(percent(100))
            .padding(px(5))
            .margin_left(auto())
            .display(Display::None)
            .border_radius(px(5))
            .justify_content(JustifyContent::Start)
            .shadow(BoxShadow::new(
                Color::BLACK.with_alpha(0.8),
                px(0.0),
                px(1.0),
                px(1.0),
                px(2.0),
            ))
            .background_color("white")
    );

    styles.bind_id(
        "console-panel",
        Style::new()
            .flex_grow(1.0)
            .height(px(240))
            .padding(px(5))
            .margin_left(px(2))
            .margin_right(px(2))
            .border_radius(px(5))
            .justify_content(JustifyContent::Start)
            .margin_top(auto())
            .shadow(BoxShadow::new(
                Color::BLACK.with_alpha(0.8),
                px(0.0),
                px(1.0),
                px(1.0),
                px(2.0),
            ))
            .background_color("white")
    );

    styles.bind_class(
        "operator-btn",
        Style::new()
            .width(percent(100))
            .justify_content(JustifyContent::Start)
            .no_shadow()
            .border_radius(px(2))
    );

    styles.bind_class(
        "property-container",
        Style::new().margin_top(px(20))
    );
}
