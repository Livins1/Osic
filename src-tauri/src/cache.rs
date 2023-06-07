
use redb::{Database, Error, TableDefinition, ReadableTable};
use redb::{RedbKey, RedbValue};

const TABLE: TableDefinition<&str, &str> = TableDefinition::new("default");

// pub struct AppCache<'txn, K: RedbKey + 'static, V: RedbValue + 'static> {
//     db: Database,
// }
pub struct AppCache {
    db: Database,
}

impl AppCache {
    pub fn new(path: &str) -> Self {
        AppCache {
            db: Database::create(path).expect("Redb: Database Create Error"),
        }
    }

    pub fn table_write(&self, key: &str, value: &str) -> Result<(), Error> {
        let write_txn = self.db.begin_write()?;
        {
            // let mut table = write_txn.open_table(TableDefinition::new(table_name))?;
            let mut table = write_txn.open_table(TABLE)?;
            table.insert(key, value)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn write_table<'a, K: RedbKey, V: RedbValue>(
        &self,
        table: TableDefinition<K, V>,
        key: K::SelfType<'a>,
        value: V::SelfType<'a>,
    ) -> Result<(), Error> {
        let write_txn = self.db.begin_write()?;
        {
            // let mut table = write_txn.open_table(TableDefinition::new(table_name))?;
            let mut t = write_txn.open_table(table)?;
            t.insert(key, value)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn save_thumbnail(&self, key: &str, value: String) -> Result<(), Error> {
        let table: TableDefinition<&str, &str> = TableDefinition::new("thumbnail");
        self.write_table(table, key, value.as_str())?;
        Ok(())
    }

    pub fn read_thumbnail(&self, key: &str) -> Result<Option<String>, Error> {
        let table: TableDefinition<&str, &str> = TableDefinition::new("thumbnail");
        let read_txn = self.db.begin_read()?;
        let t = read_txn.open_table(table)?;
        if let Some(value) = t.get(key).unwrap() {
            // value.value()
            return Ok(Some(value.value().to_string()))
        };
        // Ok(String::new())
        Ok(None)

    }

}
