use leptos::either::*;

impl<T: Sized> IntoEitherOf2 for T {}
pub trait IntoEitherOf2: Sized {
    fn into_either_of_2a<B>(self) -> Either<Self, B> {
        Either::Left(self)
    }

    fn into_either_of_2b<A>(self) -> Either<A, Self> {
        Either::Right(self)
    }
}

macro_rules! impl_into_either_of {
    (
        $(
            $trait:ident => $struct:ident {
                $($name:ident<$($generic:ident),+> -> $variant:ident<$($return:ty),+>);+;
            }
        )+
    ) => {
        $(
            impl<T: Sized> $trait for T {}
            pub trait $trait: Sized {
                $(
                    fn $name<$($generic),+>(self) -> $struct<$($return),+> {
                        $struct::$variant(self)
                    }
                )+
            }
        )+
    };
}

impl_into_either_of! {
    IntoEitherOf3 => EitherOf3 {
        into_either_of_3a<B, C> -> A<Self, B, C>;
        into_either_of_3b<A, C> -> B<A, Self, C>;
        into_either_of_3c<A, B> -> C<A, B, Self>;
    }

    IntoEitherOf4 => EitherOf4 {
        into_either_of_4a<B, C, D> -> A<Self, B, C, D>;
        into_either_of_4b<A, C, D> -> B<A, Self, C, D>;
        into_either_of_4c<A, B, D> -> C<A, B, Self, D>;
        into_either_of_4d<A, B, C> -> D<A, B, C, Self>;
    }

    IntoEitherOf5 => EitherOf5 {
        into_either_of_5a<B, C, D, E> -> A<Self, B, C, D, E>;
        into_either_of_5b<A, C, D, E> -> B<A, Self, C, D, E>;
        into_either_of_5c<A, B, D, E> -> C<A, B, Self, D, E>;
        into_either_of_5d<A, B, C, E> -> D<A, B, C, Self, E>;
        into_either_of_5e<A, B, C, D> -> E<A, B, C, D, Self>;
    }

    IntoEitherOf6 => EitherOf6 {
        into_either_of_6a<B, C, D, E, F> -> A<Self, B, C, D, E, F>;
        into_either_of_6b<A, C, D, E, F> -> B<A, Self, C, D, E, F>;
        into_either_of_6c<A, B, D, E, F> -> C<A, B, Self, D, E, F>;
        into_either_of_6d<A, B, C, E, F> -> D<A, B, C, Self, E, F>;
        into_either_of_6e<A, B, C, D, F> -> E<A, B, C, D, Self, F>;
        into_either_of_6f<A, B, C, D, E> -> F<A, B, C, D, E, Self>;
    }

    IntoEitherOf7 => EitherOf7 {
        into_either_of_7a<B, C, D, E, F, G> -> A<Self, B, C, D, E, F, G>;
        into_either_of_7b<A, C, D, E, F, G> -> B<A, Self, C, D, E, F, G>;
        into_either_of_7c<A, B, D, E, F, G> -> C<A, B, Self, D, E, F, G>;
        into_either_of_7d<A, B, C, E, F, G> -> D<A, B, C, Self, E, F, G>;
        into_either_of_7e<A, B, C, D, F, G> -> E<A, B, C, D, Self, F, G>;
        into_either_of_7f<A, B, C, D, E, G> -> F<A, B, C, D, E, Self, G>;
        into_either_of_7g<A, B, C, D, E, F> -> G<A, B, C, D, E, F, Self>;
    }
}
