use windows_reactor::*;

pub fn comparison(first: &str, second: &str) -> Element {
    vstack((
        text_block(first).font_weight(500).wrap(),
        text_block(second)
            .font_size(14.0)
            .foreground(ThemeRef::SecondaryText)
            .wrap(),
    ))
    .spacing(5.0)
    .into()
}
