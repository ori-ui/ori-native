use ori_native::prelude::*;

fn main() {
    let mut data = Data { todos: Vec::new() };

    App::new().run(&mut data, ui);
}

struct Data {
    todos: Vec<Todo>,
}

struct Remove(usize);

struct Todo {
    name: String,
    done: bool,
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
                .border(1.0)
                .border_color(Color::BLACK),
            )
            .flex(1.0)
            .justify_contents(Justify::Center)
            .align_items(Align::Center),
        ),
        receive(|data: &mut Data, Remove(index)| {
            data.todos.remove(index);
        }),
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
    let todos = data
        .todos
        .iter()
        .enumerate()
        .map(|(i, x)| todo(i, x))
        .collect::<Vec<_>>();

    column(vscroll(column(todos)))
        .max_height(400.0)
        .border_top(1.0)
        .border_color(Color::BLACK)
}

fn todo(index: usize, _todo: &Todo) -> impl View<Data> + use<> {
    let view = pressable(move |todo: &Todo, _| {
        let name = if todo.done {
            text(&todo.name)
                .color(Color::BLACK.fade(0.7))
                .strikethrough(true)
        } else {
            text(&todo.name)
        };

        row((
            done(todo),
            name.flex(1.0),
            remove(index),
        ))
        .gap(8.0)
        .padding(8.0)
        .border_top(if index > 0 { 1.0 } else { 0.0 })
        .border_color(Color::BLACK)
        .justify_contents(Justify::SpaceBetween)
    })
    .on_press(|todo: &mut Todo| todo.done = !todo.done);

    map(view, move |data: &mut Data, f| {
        f(&mut data.todos[index])
    })
}

fn done(todo: &Todo) -> impl View<Todo> + use<> {
    row(todo.done.then(|| text("x").color(Color::GREEN)))
        .size(24.0, 24.0)
        .border(1.0)
        .corners(12.0)
        .border_color(Color::BLACK)
        .justify_contents(Justify::Center)
        .align_items(Align::Center)
}

fn remove<T>(index: usize) -> impl View<T> {
    pressable(|_, _| {
        row(text("x"))
            .size(24.0, 24.0)
            .corners(8.0)
            .background_color(Color::RED)
            .justify_contents(Justify::Center)
            .align_items(Align::Center)
    })
    .on_press(move |_| Message::new(Remove(index), None))
}
