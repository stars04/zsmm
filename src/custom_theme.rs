use iced_core::{color,Color, Background, border, Border, Shadow};
use iced::Font;
use iced::font::{Family, Weight, Stretch, Style};

//TODO: Make custom theme for application
pub fn label_font() -> iced::Font {
    Font {
        family: Family::Monospace,
        weight: Weight::Bold,
        stretch: Stretch::Normal,
        style: Style::Normal,
    }
}
pub fn scroll_container_style() -> iced::widget::container::Style {
    iced::widget::container::Style{
        text_color: Some(Color::from_rgb8(255, 255, 255)),
        background: Some(Background::Color(Color::from_rgb8(98, 94, 90))),
        border: Border {
            color: Color::from_rgb8(0, 0, 0),
            width: 5.0,
            radius: border::Radius {
                ..Default::default()
            },
        },
        shadow: Shadow {
            color: Color::from_rgb8(0, 0, 0),
            offset: iced_core::Vector { x: 0.0, y: 0.0 },
            blur_radius: 0.0,
        },
    }
}
pub fn label_container_style() -> iced::widget::container::Style {
    iced::widget::container::Style{
        text_color: Some(Color::from_rgb8(255, 255, 255)),
        background: Some(Background::Color(Color::from_rgb8(98, 94, 90))),
        border: Border {
            color: Color::from_rgb8(0, 0, 0),
            width: 5.0,
            radius: border::Radius {
                ..Default::default()
            },
        },
        shadow: Shadow {
            color: Color::from_rgb8(0, 0, 0),
            offset: iced_core::Vector { x: 0.0, y: 0.0 },
            blur_radius: 0.0,
        },
    }
}
