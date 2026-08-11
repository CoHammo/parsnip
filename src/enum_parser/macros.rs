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
