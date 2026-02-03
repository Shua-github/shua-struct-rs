use std::ffi::CString;

use shua_struct::{BinaryField as _, BinaryStruct};

#[derive(Debug, Default, BinaryStruct)]
#[binary_struct(bit_order = shua_struct::Lsb0)]
pub struct Item {
    pub id: u16,
    pub quantity: u8,
    pub value: f32,
    #[binary_field(align = 8)]
    pub flags: [bool; 4],
}

#[derive(Debug, Default, BinaryStruct)]
#[binary_struct(bit_order = shua_struct::Lsb0)]
pub struct Inventory {
    pub max_slots: u8,
    #[binary_field(size_func = get_actual_slots)]
    pub items: Vec<Item>,
}

impl Inventory {
    fn get_actual_slots(&self) -> usize {
        return self.max_slots as usize;
    }
}

#[derive(Debug, Default, BinaryStruct)]
#[binary_struct(bit_order = shua_struct::Lsb0)]
pub struct Player {
    pub player_id: u32,
    pub name: CString,
    pub level: u8,
    #[binary_field(size_field = level)]
    pub skills: Vec<Skill>,
    pub inventory: Inventory,
}

#[derive(Debug, Default, BinaryStruct)]
#[binary_struct(bit_order = shua_struct::Lsb0)]
pub struct Skill {
    pub skill_id: u8,
    pub points: u8,
    pub multiplier: f32,
    #[binary_field(align = 8)]
    pub modifiers: [bool; 2],
}

#[derive(Debug, Default, BinaryStruct)]
#[binary_struct(bit_order = shua_struct::Lsb0)]
pub struct GameSave {
    pub version: u16,
    pub player_count: u8,
    #[binary_field(size_field = player_count)]
    pub players: Vec<Player>,
    #[binary_field(align = 8)]
    pub options: [bool; 6],
}

fn main() {
    let game_save = GameSave {
        version: 1,
        player_count: 2,
        options: [true, false, true, false, true, true],
        players: vec![
            Player {
                player_id: 1001,
                name: CString::new("Player One").unwrap(),
                level: 3,
                skills: vec![
                    Skill {
                        skill_id: 1,
                        points: 5,
                        multiplier: 1.2,
                        modifiers: [true, false],
                    },
                    Skill {
                        skill_id: 2,
                        points: 3,
                        multiplier: 1.0,
                        modifiers: [false, true],
                    },
                    Skill {
                        skill_id: 3,
                        points: 1,
                        multiplier: 0.8,
                        modifiers: [true, true],
                    },
                ],
                inventory: Inventory {
                    max_slots: 2,
                    items: vec![
                        Item {
                            id: 101,
                            quantity: 5,
                            value: 10.5,
                            flags: [true, false, false, true],
                        },
                        Item {
                            id: 202,
                            quantity: 1,
                            value: 100.0,
                            flags: [false, true, true, false],
                        },
                    ],
                },
            },
            Player {
                player_id: 1002,
                name: CString::new("Player Two").unwrap(),
                level: 2,
                skills: vec![
                    Skill {
                        skill_id: 1,
                        points: 2,
                        multiplier: 1.1,
                        modifiers: [true, true],
                    },
                    Skill {
                        skill_id: 2,
                        points: 4,
                        multiplier: 1.3,
                        modifiers: [false, true],
                    },
                ],
                inventory: Inventory {
                    max_slots: 1,
                    items: vec![Item {
                        id: 101,
                        quantity: 2,
                        value: 10.5,
                        flags: [true, true, false, false],
                    }],
                },
            },
        ],
    };

    let serialized = game_save.build(&None).unwrap();

    let deserialized = GameSave::parse(&serialized, &None).unwrap().0;

    let serialized_again = deserialized.build(&None).unwrap();

    println!("raw: \n{:?}\ndeserialized: \n{:?}", game_save, deserialized);

    assert_eq!(serialized, serialized_again);
}
