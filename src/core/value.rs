static OBJ_TAG: u64 = 0x8000000000000000;
static QNAN: u64 = 0x7ffc000000000000;
static NIL_TAG: u64 = 1;
static FALSE_TAG: u64 = 2;
static TRUE_TAG: u64 = 3;

#[derive(Clone, Copy)]
pub struct Value {
    bits: u64,
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_nil() {
            write!(f, "nil")
        } else if self.is_boolean() {
            write!(f, "{}", self.as_boolean())
        } else if self.is_number() {
            write!(f, "{}", self.as_number())
        } else if self.is_object() {
            write!(f, "<object:{}>", self.as_object())
        } else {
            write!(f, "<unknown>")
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            n if n.is_nil() => false,
            b if b.is_boolean() => b.as_boolean(),
            n if n.is_number() => true,
            o if o.is_object() => true,
            _ => panic!("Inavlid bit sequence for value"),
        }
    }
}

// Nil
impl Value {
    #[inline]
    pub fn nil() -> Self {
        Self {
            bits: QNAN | NIL_TAG,
        }
    }

    #[inline]
    pub fn is_nil(&self) -> bool {
        self.bits == (QNAN | NIL_TAG)
    }
}

// Boolean
impl Value {
    #[inline]
    pub fn boolean(b: bool) -> Self {
        Self {
            bits: QNAN | (if b { TRUE_TAG } else { FALSE_TAG }),
        }
    }

    #[inline]
    pub fn is_boolean(&self) -> bool {
        (self.bits | 1) == TRUE_TAG | QNAN
    }

    #[inline]
    pub fn as_boolean(&self) -> bool {
        (self.bits & TRUE_TAG) == TRUE_TAG
    }
}

// Number
impl Value {
    #[inline]
    pub fn number(n: f64) -> Self {
        Self { bits: n.to_bits() }
    }

    #[inline]
    pub fn is_number(&self) -> bool {
        (self.bits & QNAN) != QNAN
    }

    #[inline]
    pub fn as_number(&self) -> f64 {
        f64::from_bits(self.bits)
    }
}

// Object
impl Value {
    #[inline]
    pub fn object(ptr: usize) -> Self {
        Self {
            bits: OBJ_TAG | QNAN | ptr as u64,
        }
    }

    #[inline]
    pub fn is_object(&self) -> bool {
        self.bits & (QNAN | OBJ_TAG) == (QNAN | OBJ_TAG)
    }

    #[inline]
    pub fn as_object(&self) -> usize {
        (self.bits & !(QNAN | OBJ_TAG)) as usize
    }
}
