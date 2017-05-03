#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SerialResponseError{
  UnknownRequest{msg: String},
  JsonParseError{msg: String, bad_json: String},
  PortNotFound{port: String, msg: String},
  SubscriptionNotFound{sub_id: String, msg: String},
  AlreadyWriteLocked{port: String, msg: String},
  NeedWriteLock{port: String, msg: String},
  ReadError{port: String, msg: String},
  WriteError{port: String, msg: String}
}
