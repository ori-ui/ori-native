use ori_native::prelude::*;

fn main() {
    App::init_log();

    let mut data = Data { todos: Vec::new() };

    App::new().run(&mut data, ui).unwrap();
}

struct Data {
    todos: Vec<Todo>,
}

struct Remove(usize);

struct Todo {
    name: String,
    done: bool,
}

mod theme {
    use ori_native::Color;

    pub const BACKGROUND: Color = Color::hex("#1e1e1e");
    pub const BORDER: Color = Color::hex("#4d4d4d");
    pub const SUCCESS: Color = Color::hex("#9af079");
    pub const DANGER: Color = Color::hex("#f05d51");
    pub const TEXT: Color = Color::hex("#f9f9f8");
}

fn ui(data: &Data) -> impl Effect<Data> + use<> {
    effects((
        window(
            column(
                column((
                    input(),
                    (!data.todos.is_empty()).then(|| todos(data)),
                ))
                .width(300.0)
                .align_items(Align::Stretch)
                .border_width(1.0)
                .border_color(theme::BORDER),
            )
            .flex(1.0)
            .justify_content(Justify::Center)
            .align_items(Align::Center)
            .background(theme::BACKGROUND),
        ),
        receive(
            None,
            |data: &mut Data, Remove(index)| {
                data.todos.remove(index);
            },
        ),
    ))
}

fn input() -> impl View<Data> + use<> {
    with(
        |_| String::new(),
        |name, _| {
            row(textinput()
                .text(name)
                .placeholder("What do you want to do?")
                .newline(Newline::None)
                .accept_tab(false)
                .color(theme::TEXT)
                .placeholder_color(theme::TEXT.fade(0.6))
                .on_change(|(name, _), text| *name = text)
                .on_submit(|(name, data), text| {
                    add_todo(data, text);
                    name.clear();
                })
                .flex(1.0))
            .padding(8.0)
        },
    )
}

fn add_todo(data: &mut Data, name: String) {
    // add a todo to the top of the list
    data.todos.insert(0, Todo { name, done: false })
}

fn todos(data: &Data) -> impl View<Data> + use<> {
    column(list(
        data.todos.len(),
        |data: &Data, i| todo(i, &data.todos[i]),
    ))
    .max_height(400.0)
    .border_top_width(1.0)
    .border_color(theme::BORDER)
}

fn todo(index: usize, _todo: &Todo) -> impl View<Data> + use<> {
    let view = pressable(move |todo: &Todo, state| {
        let name = if todo.done {
            text(&todo.name)
                .color(theme::TEXT.fade(0.6))
                .strikethrough(true)
        } else {
            text(&todo.name).color(theme::TEXT)
        };

        row((
            done(todo),
            name.flex(1.0),
            remove(index),
        ))
        .gap(8.0)
        .padding(8.0)
        .border_top_width(if index > 0 { 1.0 } else { 0.0 })
        .border_color(theme::BORDER)
        .justify_content(Justify::SpaceBetween)
        .align_items(Align::Center)
        .background(if state.hovered {
            theme::BACKGROUND.lighten(0.02)
        } else {
            Color::TRANSPARENT
        })
        .min_width(0.0)
    })
    .on_press(|todo: &mut Todo, _| todo.done = !todo.done);

    map(view, move |data: &mut Data, map| {
        map(&mut data.todos[index])
    })
}

fn done(todo: &Todo) -> impl View<Todo> + use<> {
    let icon = (todo.done).then(|| image(include_bytes!("check.svg")).tint(theme::SUCCESS));

    row(icon)
        .size(28.0, 28.0)
        .border_width(1.0)
        .corner(14.0)
        .border_color(theme::BORDER)
        .justify_content(Justify::Center)
        .align_items(Align::Center)
}

fn remove<T>(index: usize) -> impl View<T> {
    pressable(|_, _| {
        let icon = image(include_bytes!("xmark.svg")).tint(theme::TEXT);

        row(icon)
            .size(28.0, 28.0)
            .corner(8.0)
            .background(theme::DANGER)
            .justify_content(Justify::Center)
            .align_items(Align::Center)
    })
    .on_press(move |_, _| Message::new(Remove(index), None))
}
