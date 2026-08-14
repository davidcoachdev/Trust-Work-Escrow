use trust_escrow_sdk::error::Result as SdkResult;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateJobRequest { pub title: String, pub description: String, pub amount: u64 }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationResult { pub signature: String }

pub trait SdkBoundary {
    fn create_job(&self, request: &CreateJobRequest) -> Result<String, String>;
}

pub struct ApplicationService<S> { sdk: S }

impl<S: SdkBoundary> ApplicationService<S> {
    pub fn new(sdk: S) -> Self { Self { sdk } }
    pub fn create_job(&self, request: CreateJobRequest) -> Result<OperationResult, String> {
        if request.title.trim().is_empty() { return Err("title is required".into()); }
        if request.amount == 0 { return Err("amount must be positive".into()); }
        self.sdk.create_job(&request).map(|signature| OperationResult { signature })
    }
}

// Keeps the SDK error category visible at this boundary without importing RPC
// types into the API. Concrete adapters may use this conversion.
#[allow(dead_code)]
fn _sdk_result<T>(result: SdkResult<T>) -> Result<T, String> { result.map_err(|error| error.to_string()) }
