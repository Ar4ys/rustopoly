use leptos::prelude::*;
use tailwind_merge::tw;

type Rotation<'a> = (&'a str, &'a str);

const SIDE: [(Rotation, Rotation); 6] = [
    /*  1  */
    (("--rotateY-deg", "0deg"), ("--rotateX-deg", "0deg")),
    /*  2  */
    (("--rotateY-deg", "270deg"), ("--rotateX-deg", "0deg")),
    /*  3  */
    (("--rotateY-deg", "0deg"), ("--rotateX-deg", "90deg")),
    /*  4  */
    (("--rotateY-deg", "0deg"), ("--rotateX-deg", "270deg")),
    /*  5  */
    (("--rotateY-deg", "90deg"), ("--rotateX-deg", "0deg")),
    /*  6  */
    (("--rotateY-deg", "0deg"), ("--rotateX-deg", "180deg")),
];

/// Non-reactive component.
#[component]
pub fn Dice(
    #[prop(into, optional)] class: String,
    side: usize,
    #[prop(optional)] animated: bool,
) -> impl IntoView {
    assert!(
        (1..=6).contains(&side),
        "Dice has only 6 sides, silly. Received {}",
        side
    );

    view! {
        <div
            class=tw!("w-40 aspect-square [perspective:1000px]", class)
            style=SIDE[side - 1].0
            style=SIDE[side - 1].1
        >

            {(1..=6).map(|face| dice_face(face, animated)).collect_view()}
        </div>
    }
}

fn dice_face(face: usize, animated: bool) -> impl IntoView {
    view! {
        <div
            style:animation-name=format!("dice-surface-{face}")
            class=tw!(
                "flex overflow-hidden absolute justify-center items-center w-full bg-white select-none",
                "aspect-square origin-[50%_50%_-5rem] border-[0.2rem] text-[6rem] [backface-visibility:hidden]",
                if animated { "animate-[0.7s_ease-in-out_forwards]" } else { "animate-[0s_forwards]" }
            )
        >
            <div
                class="grid gap-[0.6rem]"
                style:grid-template-columns=match face {
                    2 | 4 | 6 => "repeat(2, 1fr)",
                    3 | 5 => "repeat(3, 1fr)",
                    _ => "",
                }
            >
                {(1..=face).map(|dot| dice_dot(face, dot)).collect_view()}
            </div>
        </div>
    }
}

fn dice_dot(face: usize, dot: usize) -> impl IntoView {
    let (column, row) = match face {
        /*
        -----
        |   |
        | o |
        |   |
        -----
        */
        1 => match dot {
            1 => (1, 1),
            _ => unreachable!(),
        },

        /*
        -----
        |  o|
        |   |
        |o  |
        -----
        */
        2 => match dot {
            1 => (2, 1),
            2 => (1, 2),
            _ => unreachable!(),
        },

        /*
        -----
        |  o|
        | o |
        |o  |
        -----
        */
        3 => match dot {
            1 => (3, 1),
            2 => (2, 2),
            3 => (1, 3),
            _ => unreachable!(),
        },

        /*
        -----
        |o o|
        |   |
        |o o|
        -----
        */
        4 => match dot {
            1 => (1, 1),
            2 => (2, 1),
            3 => (1, 2),
            4 => (2, 2),
            _ => unreachable!(),
        },

        /*
        -----
        |o o|
        | o |
        |o o|
        -----
        */
        5 => match dot {
            1 => (1, 1),
            2 => (3, 1),
            3 => (2, 2),
            4 => (1, 3),
            5 => (3, 3),
            _ => unreachable!(),
        },

        /*
        -----
        |o o|
        |o o|
        |o o|
        -----
        */
        6 => match dot {
            1 => (1, 1),
            2 => (2, 1),
            3 => (1, 2),
            4 => (2, 2),
            5 => (1, 3),
            6 => (2, 3),
            _ => unreachable!(),
        },

        _ => unreachable!(),
    };

    view! {
        <div
            class="w-8 bg-black rounded-full aspect-square"
            style:grid-column=column.to_string()
            style:grid-row=row.to_string()
        />
    }
}

// Excuse me WTF IS THIS FORMATTING?!
// Ths stupidity resolves, if I remove "{}".
// TODO: Fix this and PR into leptosfmt.
// #[component]
// pub fn Dice() -> impl IntoView {
//     view! {
//         <div class="w-40 aspect-square [perspective:1000px]">
//             {} <div class="surface">
//                 <div class="dots">
//                     <div class="dot"></div>
//                 </div>
//             </div> <div class="surface">
//                 <div class="dots">
//                     <div class="dot bg-transparent"></div>
//                     <div class="dot"></div>
//                     <div class="dot"></div>
//                     <div class="dot bg-transparent"></div>
//                 </div>
//             </div>
//         </div>
//     }
// }
