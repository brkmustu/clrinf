namespace MinimalService;

public sealed record RequestContext(string CorrelationId);
public sealed record ApplicationError(string Code, string Message);

public abstract record Result<T>
{
    private Result() { }
    public sealed record Success(T Value) : Result<T>;
    public sealed record Failure(ApplicationError Error) : Result<T>;
}

public interface IApplicationModule<in TRequest, TResponse>
{
    Result<TResponse> Handle(TRequest request, RequestContext context);
}

public static class Dispatcher
{
    public static Result<TResponse> Dispatch<TRequest, TResponse>(
        IApplicationModule<TRequest, TResponse> module,
        TRequest request,
        RequestContext context) => module.Handle(request, context);
}
