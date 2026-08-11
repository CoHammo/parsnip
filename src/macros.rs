/// Macro for generating a parser struct with the given name and fields.
// #[macro_export]
// macro_rules! parser {
//     (
//         $name:ident $func_name:ident
//         { $( $fields:tt )* }
//         { $( $init:tt )* }
//     ) => {
//         parser!(@munch
//             { $( $fields )* }
//             $name $func_name { }
//             newargs()
//             { $( $init )* }
//             defs()
//         );
//     };

//     (@munch
//         { $field:ident: $ft:ty, $( $left:tt )* }
//         $name:ident $func_name:ident { $( $fields:tt )* }
//         newargs( $( $args:tt )* )
//         { $( $init:tt )* }
//         defs( $( $defs:tt )* )
//     ) => {
//         parser!(@munch
//             { $( $left )* }
//             $name $func_name { $( $fields )* pub $field: $ft, }
//             newargs( $( $args )* $field: $ft, )
//             { $( $init )* }
//             defs( $( $defs )* $field, )
//         );
//     };

//     (@munch
//         { => $field:ident: $ft:ty, $( $left:tt )* }
//         $name:ident $func_name:ident { $( $fields:tt )* }
//         newargs( $( $args:tt )* )
//         { $( $init:tt )* }
//         defs( $( $defs:tt )* )
//     ) => {
//         parser!(@munch
//             { $( $left )* }
//             $name $func_name { $( $fields )* pub $field: $ft, }
//             newargs( $( $args )* )
//             {
//                 let $field: $ft;
//                 $( $init )*
//             }
//             defs( $( $defs )* $field, )
//         );
//     };

//     (@munch
//         { $arg:ident: $at:ty => $field:ident: $ft:ty, $( $left:tt )* }
//         $name:ident $func_name:ident { $( $fields:tt )* }
//         newargs( $( $args:tt )* )
//         { $( $init:tt )* }
//         defs( $( $defs:tt )* )
//     ) => {
//         parser!(@munch
//             { $( $left )* }
//             $name $func_name { $( $fields )* pub $field: $ft, }
//             newargs( $( $args )* $arg: $at, )
//             {
//                 let $field: $ft;
//                 $( $init )*
//             }
//             defs( $( $defs )* $field, )
//         );
//     };

//     (@munch
//         { $field:ident: $ft:ty = $default:expr, $( $left:tt )* }
//         $name:ident $func_name:ident { $( $fields:tt )* }
//         newargs( $( $args:tt )* )
//         { $( $init:tt )* }
//         defs( $( $defs:tt )* )
//     ) => {
//         parser!(@munch
//             { $( $left )* }
//             $name $func_name { $( $fields )* pub $field: $ft, }
//             newargs( $( $args )* )
//             { $( $init )* }
//             defs( $( $defs )* $field: $default, )
//         );
//     };

//     (@munch
//         { }
//         $name:ident $func_name:ident { $( $fields:tt )* }
//         newargs( $( $arg:ident: $at:ty, )* )
//         { $( $init:tt )* }
//         defs( $( $defs:tt )* )
//     ) => {
//         #[derive(Debug)]
//         pub struct $name {
//             pub base: BaseParser,
//             $( $fields )*
//         }

//         impl $name {
//             pub fn new( $( $arg: $at, )* ) -> Self {
//                 $( $init )*
//                 Self {
//                     base: BaseParser::new(),
//                     $( $defs )*
//                 }
//             }
//         }

//         pub fn $func_name($( $arg: $at, )* ) -> Parser {
//             Parser::$name($name::new($( $arg, )*))
//         }
//     };
// }

#[macro_export]
macro_rules! dispatch {
    ($name:ident) => {
        impl<T: Matches + 'static> ParserDispatch<T> for $name<T> {
            // fn get_base_fn(&self) -> fn(me: &mut dyn Any) -> &mut Base {
            //     fn func<I: Matches + 'static>(me: &mut dyn Any) -> &mut Base {
            //         unsafe {
            //             let ptr = &mut *(me as *mut dyn Any as *mut $name<I>);
            //             ptr.base()
            //         }
            //     }
            //     func::<T>
            // }

            fn get_snip_fn(
                &self,
            ) -> fn(base: *mut ParserWrap<T>, me: &mut dyn Any, snip: &Snip<T>) {
                fn func<I: Matches + 'static>(
                    base: *mut ParserWrap<I>,
                    me: &mut dyn Any,
                    snip: &Snip<I>,
                ) {
                    unsafe {
                        let ptr = &mut *(me as *mut dyn Any as *mut $name<I>);
                        ptr.snip(&mut *base, snip)
                    }
                }
                func::<T>
            }

            fn get_finish_fn(
                &self,
            ) -> fn(base: *mut ParserWrap<T>, me: &mut dyn Any, snip: &Snip<T>) {
                fn func<I: Matches + 'static>(
                    base: *mut ParserWrap<I>,
                    me: &mut dyn Any,
                    snip: &Snip<I>,
                ) {
                    unsafe {
                        let ptr = &mut *(me as *mut dyn Any as *mut $name<I>);
                        ptr.finish(&mut *base, snip)
                    }
                }
                func::<T>
            }

            fn get_reset_fn(&self) -> fn(me: &mut dyn Any) {
                fn func<I: Matches + 'static>(me: &mut dyn Any) {
                    unsafe {
                        let ptr = &mut *(me as *mut dyn Any as *mut $name<I>);
                        ptr.reset()
                    }
                }
                func::<T>
            }

            fn get_string_fn(&self) -> fn(me: &dyn Any) -> String {
                fn func<I: Matches + 'static>(me: &dyn Any) -> String {
                    unsafe {
                        let p = &*(me as *const dyn Any as *const $name<I>);
                        p.string()
                    }
                }
                func::<T>
            }
        }
    };
}

#[macro_export]
macro_rules! freshen {
    ($self:ident, $item:expr $(, $more:expr )?) => {
        if $self.base.fresh {
            $self.base.start = $item.index();
            $( $more )?
            $self.base.fresh = false;
        }
    };
}

/// Macro for implementing the parser methods for a given set of parser variants.
#[macro_export]
macro_rules! parser_enum {
    ($($variant:ident),*) => {
        #[derive(Debug, Default, Clone)]
        pub enum Parser<T: PItem> {
            #[default]
            Default,
            $($variant($variant<T>)),*
        }

        impl<T: PItem> Parser<T> {
            fn fresh(&self) -> bool {
                match self {
                    Self::Default => false,
                    $(Self::$variant(p) => p.base.fresh,)*
                }
            }

            fn start(&self) -> usize {
                match self {
                    Self::Default => 0,
                    $(Self::$variant(p) => p.base.start,)*
                }
            }

            pub fn string(&self) -> String {
                match self {
                    Self::Default => "Parser()".to_string(),
                    $(Self::$variant(p) => p.string(),)*
                }
            }

            pub fn parse(&mut self, parses: &impl Parses<T>, range: impl RangeBounds<usize>) -> Stat {
                match self {
                    Self::Default => Stat::Failed,
                    $(Self::$variant(p) => {
                        let mut iter = parses.to_parse_iter(range);
                        while let Some(item) = iter.next() {
                            match p.take(&item) {
                                Stat::Matched(_) | Stat::Failed => break,
                                _ => {},
                            }
                        }
                        return match p.base.stat {
                            Stat::Running => p.finish(&iter.item()),
                            _ => p.base.stat,
                        }
                    },)*
                }
            }

            pub fn take_tokens(&mut self) -> Option<Vec<Token>> {
                match self {
                    Self::Default => None,
                    $(Self::$variant(p) => p.base.tokens.take(),)*
                }
            }

            fn take<I: Iterator<Item = T> + Clone>(&mut self, byte: &ParseItem<T, I>) -> Stat {
                match self {
                    Self::Default => Stat::Failed,
                    $(Self::$variant(p) => p.take(byte),)*
                }
            }

            fn finish<I: Iterator<Item = T> + Clone>(&mut self, byte: &ParseItem<T, I>) -> Stat {
                match self {
                    Self::Default => Stat::Failed,
                    $(Self::$variant(p) => p.finish(byte),)*
                }
            }

            fn reset(&mut self) {
                match self {
                    Self::Default => {},
                    $(Self::$variant(p) => p.reset(),)*
                }
            }

            pub fn tokens_str(&mut self) -> String {
                match self {
                    Self::Default => "".to_string(),
                    $(Self::$variant(p) => format!("{:#?}", p.base.tokens),)*
                }
            }
        }

        // impl<T: Pit> Clone for Parser<T> {
        //     fn clone(&self) -> Self {
        //         match self {
        //             Self::Default => Self::Default,
        //             $(Self::$variant(p) => Self::$variant(p.clone()),)*
        //         }
        //     }
        // }
    };
}
