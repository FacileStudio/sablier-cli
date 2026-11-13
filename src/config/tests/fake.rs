use crate::vault::Vault;

pub struct FakeVault {
    available: bool,
    keeps: bool,
    pub token: std::cell::RefCell<Option<String>>,
}

impl FakeVault {
    pub fn new(available: bool, keeps: bool) -> Self {
        Self {
            available,
            keeps,
            token: std::cell::RefCell::new(None),
        }
    }

    fn up(&self) -> Result<(), String> {
        if self.available {
            return Ok(());
        }
        Err("no secret service".into())
    }
}

impl Vault for FakeVault {
    fn get(&self) -> Result<Option<String>, String> {
        self.up()?;
        Ok(self.token.borrow().clone())
    }

    fn set(&self, token: &str) -> Result<(), String> {
        self.up()?;
        if self.keeps {
            *self.token.borrow_mut() = Some(token.to_string());
        }
        Ok(())
    }

    fn delete(&self) -> Result<bool, String> {
        self.up()?;
        Ok(self.token.borrow_mut().take().is_some())
    }
}
