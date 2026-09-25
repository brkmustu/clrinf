use crate::core::{ApplicationError, ApplicationModule, RequestContext};

pub struct Greet {
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct Greeting {
    pub message: String,
    pub correlation_id: String,
}

pub struct GreetingModule;

impl ApplicationModule<Greet> for GreetingModule {
    type Response = Greeting;

    fn handle(
        &self,
        request: Greet,
        context: &RequestContext,
    ) -> Result<Greeting, ApplicationError> {
        let name = request.name.trim();
        if name.is_empty() {
            return Err(ApplicationError {
                code: "validation.name_required",
                message: "Name must not be blank.",
            });
        }

        Ok(Greeting {
            message: format!("Hello, {name}!"),
            correlation_id: context.correlation_id.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::dispatch;

    #[test]
    fn dispatches_and_propagates_context() {
        let context = RequestContext {
            correlation_id: "test-request".into(),
        };
        let result = dispatch(
            &GreetingModule,
            Greet {
                name: " Ada ".into(),
            },
            &context,
        );
        assert_eq!(
            result,
            Ok(Greeting {
                message: "Hello, Ada!".into(),
                correlation_id: "test-request".into(),
            })
        );
    }

    #[test]
    fn rejects_blank_names() {
        let context = RequestContext {
            correlation_id: "test-request".into(),
        };
        for name in ["", " \t\n"] {
            let error =
                dispatch(&GreetingModule, Greet { name: name.into() }, &context).unwrap_err();
            assert_eq!(error.code, "validation.name_required");
        }
    }
}
