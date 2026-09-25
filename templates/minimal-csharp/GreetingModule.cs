namespace MinimalService;

public sealed record Greet(string? Name);
public sealed record Greeting(string Message, string CorrelationId);

public sealed class GreetingModule : IApplicationModule<Greet, Greeting>
{
    public Result<Greeting> Handle(Greet request, RequestContext context)
    {
        if (string.IsNullOrWhiteSpace(request.Name))
        {
            return new Result<Greeting>.Failure(
                new ApplicationError("validation.name_required", "Name must not be blank."));
        }

        return new Result<Greeting>.Success(
            new Greeting($"Hello, {request.Name.Trim()}!", context.CorrelationId));
    }
}
