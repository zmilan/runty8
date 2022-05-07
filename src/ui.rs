pub mod button;
pub mod cursor;
pub mod text;
use crate::{
    runtime::{cmd::Cmd, draw_context::DrawContext},
    Event,
};
use std::{fmt::Debug, marker::PhantomData};

pub struct DispatchEvent<'a, Msg> {
    queue: &'a mut Vec<Msg>,
}

impl<'a, Msg> DispatchEvent<'a, Msg> {
    pub fn new(queue: &'a mut Vec<Msg>) -> Self {
        Self { queue }
    }

    pub fn call(&mut self, msg: Msg) {
        self.queue.push(msg);
    }
}

pub trait Widget {
    type Msg: Copy + Debug;

    fn on_event(
        &mut self,
        event: Event,
        cursor_position: (i32, i32),
        dispatch_event: &mut DispatchEvent<Self::Msg>,
    );

    fn draw(&self, draw: &mut DrawContext);
}

pub struct Tree<'a, Msg> {
    children: Vec<Element<'a, Msg>>,
}

pub struct Element<'a, Msg> {
    widget: Box<dyn Widget<Msg = Msg> + 'a>,
}

impl<'a, Msg> Element<'a, Msg> {
    fn new(widget: impl Widget<Msg = Msg> + 'a) -> Self {
        Self {
            widget: Box::new(widget),
        }
    }

    pub fn as_widget(&self) -> &dyn Widget<Msg = Msg> {
        self.widget.as_ref()
    }

    pub fn as_widget_mut(&mut self) -> &mut dyn Widget<Msg = Msg> {
        self.widget.as_mut()
    }
}

impl<'a, Msg> Tree<'a, Msg> {
    pub fn new() -> Self {
        Self::with_children(vec![])
    }

    pub fn with_children(children: Vec<Element<'a, Msg>>) -> Self {
        Self { children }
    }

    pub fn push(mut self, element: impl Into<Element<'a, Msg>>) -> Self {
        self.children.push(element.into());
        self
    }
}

impl<'a, Msg: Copy + Debug> Widget for Tree<'a, Msg> {
    type Msg = Msg;

    fn on_event(
        &mut self,
        event: Event,
        cursor_position: (i32, i32),
        dispatch_event: &mut DispatchEvent<Self::Msg>,
    ) {
        for element in self.children.iter_mut() {
            element
                .widget
                .on_event(event, cursor_position, dispatch_event);
        }
    }

    fn draw(&self, draw: &mut DrawContext) {
        for element in self.children.iter() {
            element.widget.draw(draw);
        }
    }
}

pub struct DrawFn<'a, Msg> {
    pd: PhantomData<Msg>,
    f: Box<dyn Fn(&mut DrawContext) + 'a>,
}

impl<'a, Msg: Copy + Debug + 'a> DrawFn<'a, Msg> {
    pub fn new(f: impl Fn(&mut DrawContext) + 'a) -> Self {
        Self {
            f: Box::new(f),
            pd: PhantomData,
        }
    }
}

impl<'a, Msg: Copy + Debug> Widget for DrawFn<'a, Msg> {
    type Msg = Msg;

    fn on_event(
        &mut self,
        _event: Event,
        _cursor_position: (i32, i32),
        _dispatch_event: &mut DispatchEvent<Self::Msg>,
    ) {
    }

    fn draw(&self, draw: &mut DrawContext) {
        (self.f)(draw);
    }
}

impl<'a, Msg: Copy + Debug + 'a, T: Widget<Msg = Msg> + 'a> From<T> for Element<'a, Msg> {
    fn from(val: T) -> Self {
        Element::new(val)
    }
}

pub trait ElmApp2
where
    Self: Sized,
{
    type Msg: Copy + Debug;
    type Flags;

    fn init(flags: Self::Flags) -> (Self, Cmd<Self::Msg>);

    fn update(&mut self, msg: &Self::Msg) -> Cmd<Self::Msg>;

    fn view(&mut self) -> Element<'_, Self::Msg>;

    fn subscriptions(&self) -> Sub<Self::Msg>;
}

pub enum Sub<Msg> {
    OnAnimationFrame(fn(f32) -> Msg),
    NoSub,
}
