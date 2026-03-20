//! Reusable tag chip widget: colored pill with label.
//! Used in: folder list (display-only), file name panel (drag + checkbox), tag list (checkbox only).
//! Modes: display-only, with optional leading (e.g. checkbox), optional drag (lift + shadow), optional on_select.

use crate::theme;
use iced::widget::{column, container, mouse_area, row, space, stack, text};
use iced::{mouse, Alignment, Background, Element, Length, Shadow, Vector};

/// Default height of a tag chip when used as a simple pill (folder list, tag list row).
pub const CHIP_ROW_HEIGHT: f32 = 28.0;

/// Padding around chip content: [vertical, horizontal]. Chip width = horizontal padding + content + horizontal padding.
pub const CHIP_PADDING_VERTICAL: f32 = 4.0;
pub const CHIP_PADDING_HORIZONTAL: f32 = 6.0;

/// Fixed size of the leading slot (e.g. checkbox). Space is always reserved when leading is present so width never pops.
pub const LEADING_SLOT_WIDTH: f32 = 16.0;
pub const LEADING_SLOT_HEIGHT: f32 = 16.0;
/// Space between leading slot and label.
pub const LEADING_TO_LABEL_SPACING: f32 = 6.0;

/// Fixed size of the trailing slot (e.g. star + delete/save icon). Holds two icons: star (20px) + action (16px).
pub const TRAILING_SLOT_WIDTH: f32 = 36.0;
/// Space between label and trailing slot.
pub const LABEL_TO_TRAILING_SPACING: f32 = 4.0;

/// Renders a display-only tag chip: colored pill with label. No interactions, no leading content.
/// Use in the folder list (file name display).
pub fn view_display_only<Message: 'static>(
    tag_name: impl Into<String>,
    tag_color: iced::Color,
) -> Element<'static, Message> {
    let label = text(tag_name.into())
        .size(14)
        .color(iced::Color::from_rgb(0.0, 0.0, 0.0));
    let chip_inner = container(label)
        .padding([CHIP_PADDING_VERTICAL, CHIP_PADDING_HORIZONTAL]);
    let style = move |_theme: &_| iced::widget::container::Style {
        background: Some(Background::Color(tag_color)),
        border: iced::border::rounded(2),
        ..Default::default()
    };
    container(chip_inner)
        .width(Length::Shrink)
        .height(Length::Fixed(CHIP_ROW_HEIGHT))
        .style(style)
        .into()
}

/// Renders a tag chip with optional leading content (e.g. checkbox), optional drag styling (lift + shadow),
/// and optional on_select (mouse_area when selectable). Use in file name panel (leading + drag + on_select)
/// and tag list (leading only, no drag, on_select for toggle).
///
/// * `row_height` – height of the chip bar
/// * `leading` – optional content inside the chip to the left of the label (e.g. checkbox)
/// * `drag_config` – when `Some((cell_height, is_dragging))`, apply lift column and drag shadow
/// * `on_select` – when `Some(msg)`, chip is selectable (mouse_area) and pressing emits the message
/// * `leading_visible_on_hover_only` – when true, leading is visible only when `is_hovered`; space is always reserved (visibility, not collapse)
/// * `is_hovered` – when `leading_visible_on_hover_only` is true, leading is visible only when this is true
/// * `trailing` – optional content to the right of the label (e.g. delete/save icon in tag list); use with `trailing_visible_on_selection_only`
/// * `trailing_visible_on_selection_only` – when true, trailing is visible only when `is_selected`; space is always reserved (visibility, not collapse)
/// * `is_selected` – when true, trailing (if selection-only) is visible and the chip is drawn with the tag selection outline (bright blue border)
pub fn view_with_leading<Message: Clone + 'static>(
    tag_name: impl Into<String>,
    tag_color: iced::Color,
    row_height: f32,
    leading: Option<Element<'static, Message>>,
    drag_config: Option<(f32, bool)>,
    on_select: Option<Message>,
    leading_visible_on_hover_only: bool,
    is_hovered: bool,
    trailing: Option<Element<'static, Message>>,
    trailing_visible_on_selection_only: bool,
    is_selected: bool,
) -> Element<'static, Message> {
    let tag_name = tag_name.into();
    let leading_visible = !leading_visible_on_hover_only || is_hovered;
    let trailing_visible = !trailing_visible_on_selection_only || is_selected;

    let label = text(tag_name)
        .size(14)
        .color(iced::Color::from_rgb(0.0, 0.0, 0.0));

    let leading_slot: Option<Element<'_, Message>> = leading.map(|leading_el| {
        if leading_visible {
            let size_anchor = container(space())
                .width(Length::Fixed(LEADING_SLOT_WIDTH))
                .height(Length::Fixed(LEADING_SLOT_HEIGHT));
            let content = container(leading_el)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .clip(true);
            stack([size_anchor.into(), content.into()])
                .width(Length::Fixed(LEADING_SLOT_WIDTH))
                .height(Length::Fixed(LEADING_SLOT_HEIGHT))
                .into()
        } else {
            container(space())
                .width(Length::Fixed(LEADING_SLOT_WIDTH))
                .height(Length::Fixed(LEADING_SLOT_HEIGHT))
                .into()
        }
    });

    let trailing_slot: Option<Element<'_, Message>> = trailing.map(|trailing_el| {
        let slot_height = Length::Fixed(row_height);
        if trailing_visible {
            let size_anchor = container(space())
                .width(Length::Fixed(TRAILING_SLOT_WIDTH))
                .height(Length::Fixed(row_height));
            let content = container(trailing_el)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .clip(true);
            stack([size_anchor.into(), content.into()])
                .width(Length::Fixed(TRAILING_SLOT_WIDTH))
                .height(Length::Fixed(row_height))
                .into()
        } else {
            container(space())
                .width(Length::Fixed(TRAILING_SLOT_WIDTH))
                .height(slot_height)
                .into()
        }
    });

    let label_spacer = container(space())
        .width(Length::Fixed(LEADING_TO_LABEL_SPACING))
        .height(Length::Fixed(1.0));
    let trail_spacer = container(space())
        .width(Length::Fixed(LABEL_TO_TRAILING_SPACING))
        .height(Length::Fixed(1.0));

    let row_content: Element<'_, Message> = match (leading_slot, trailing_slot) {
        (Some(lead), Some(trail)) => row![lead, label_spacer, label, trail_spacer, trail]
            .spacing(0)
            .align_y(Alignment::Center)
            .width(Length::Shrink)
            .into(),
        (Some(lead), None) => row![lead, label_spacer, label]
            .spacing(0)
            .align_y(Alignment::Center)
            .width(Length::Shrink)
            .into(),
        (None, Some(trail)) => row![label, trail_spacer, trail]
            .spacing(0)
            .align_y(Alignment::Center)
            .width(Length::Shrink)
            .into(),
        (None, None) => label.into(),
    };

    let chip_inner = container(row_content)
        .padding([CHIP_PADDING_VERTICAL, CHIP_PADDING_HORIZONTAL])
        .width(Length::Shrink);

    let is_dragging = drag_config.as_ref().map(|(_, d)| *d).unwrap_or(false);
    let chip_style = move |_theme: &_| {
        let shadow = if is_dragging {
            Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 10.0,
            }
        } else {
            Shadow::default()
        };
        let border = if is_selected {
            iced::Border {
                width: 2.0,
                color: theme::ACCENT,
                radius: 4.0.into(),
            }
        } else {
            iced::border::rounded(2)
        };
        iced::widget::container::Style {
            background: Some(Background::Color(tag_color)),
            border,
            shadow,
            ..Default::default()
        }
    };

    let chip = container(chip_inner)
        .width(Length::Shrink)
        .height(Length::Fixed(row_height))
        .style(chip_style);
    let chip: Element<'_, Message> = match on_select {
        Some(msg) => mouse_area(chip)
            .on_press(msg)
            .interaction(mouse::Interaction::Pointer)
            .into(),
        None => chip.into(),
    };

    match drag_config {
        Some((cell_height, is_dragging)) => {
            let lift_px = cell_height - row_height;
            let col = if is_dragging {
                column![
                    chip,
                    container(space()).height(Length::Fixed(lift_px)),
                ]
            } else {
                column![
                    container(space()).height(Length::Fixed(lift_px)),
                    chip,
                ]
            };
            col.into()
        }
        None => chip.into(),
    }
}
