#[derive(Debug, PartialEq)]
pub struct RequestContext {
    pub correlation_id: String,
}

#[derive(Debug, PartialEq)]
pub struct ApplicationError {
    pub code: &'static str,
    pub message: &'static str,
}

pub trait ApplicationModule<Request> {
    type Response;

    fn handle(
        &self,
        request: Request,
        context: &RequestContext,
    ) -> Result<Self::Response, ApplicationError>;
}

pub fn dispatch<Request, Module: ApplicationModule<Request>>(
    module: &Module,
    request: Request,
    context: &RequestContext,
) -> Result<Module::Response, ApplicationError> {
    module.handle(request, context)
}
