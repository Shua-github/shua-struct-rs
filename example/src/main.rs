use shua_struct::BinaryField;
use std::ffi::CString;

#[derive(Debug, Default, BinaryField)]
#[binary_struct(bit_order = shua_struct::Lsb0)]
pub struct Item {
    pub id: u16,
    pub count: u8,
    #[binary_field(align = 8)]
    pub flags: [bool; 3],
}

#[derive(Debug, Default, BinaryField)]
#[binary_struct(bit_order = shua_struct::Lsb0)]
pub struct Inventory {
    pub slot_count: u8,
    #[binary_field(size_func = actual_slots)]
    pub items: Vec<Item>,
}

impl Inventory {
    fn actual_slots(&self) -> usize {
        self.slot_count as usize
    }
}

#[derive(Debug, Default, BinaryField)]
#[binary_struct(bit_order = shua_struct::Lsb0)]
pub struct Player {
    #[binary_field(check_func = check_version)]
    pub version: u8,
    pub id: u32,
    pub name: CString,
    pub level: u8,
    #[binary_field(size_field = level)]
    pub inventory: Inventory,
    #[binary_field(if_func = should_nickname)]
    pub nickname: Option<CString>,
}

impl Player {
    fn check_version(&self) -> Option<String> {
        if self.version <= 1 {
            Some("Version must be greater than 1".to_string())
        } else {
            None
        }
    }

    fn should_nickname(&self) -> bool {
        self.version >= 3
    }
}

fn main() {
    println!("=== Test 1: Version 2 (nickname skipped) ===");
    let player_v2 = Player {
        version: 2,
        id: 1,
        name: CString::new("Alice").unwrap(),
        level: 2,
        nickname: Some(CString::new("AAA").unwrap()),
        inventory: Inventory {
            slot_count: 2,
            items: vec![
                Item {
                    id: 100,
                    count: 3,
                    flags: [true, false, true],
                },
                Item {
                    id: 200,
                    count: 1,
                    flags: [false, true, false],
                },
            ],
        },
    };

    let data = player_v2.build(&None).unwrap();
    let (parsed, _) = Player::parse(&data, &None).unwrap();
    assert_eq!(parsed.nickname, None);
    let data2 = parsed.build(&None).unwrap();
    assert_eq!(data, data2);
    println!("✓ Test 1 passed\n");

    println!("=== Test 2: Version 3 (nickname included) ===");
    let player_v3 = Player {
        version: 3,
        id: 2,
        name: CString::new("Bob").unwrap(),
        level: 3,
        nickname: Some(CString::new("B-Man").unwrap()),
        inventory: Inventory {
            slot_count: 1,
            items: vec![Item {
                id: 300,
                count: 5,
                flags: [true, true, false],
            }],
        },
    };

    let data = player_v3.build(&None).unwrap();
    let (parsed, _) = Player::parse(&data, &None).unwrap();
    let data2 = parsed.build(&None).unwrap();

    assert_eq!(data, data2);
    println!("✓ Test 2 passed\n");

    println!("=== Test 3: Version 1 (invalid version, should error) ===");
    let player_v1 = Player {
        version: 1,
        id: 3,
        name: CString::new("Charlie").unwrap(),
        level: 1,
        nickname: None,
        inventory: Inventory {
            slot_count: 0,
            items: vec![],
        },
    };

    let data = player_v1.build(&None).unwrap();
    match Player::parse(&data, &None) {
        Ok(_) => println!("✗ Test 3 failed: should have returned error"),
        Err(e) => {
            println!("✓ Test 3 passed: got expected error: {}", e);
        }
    }


}
