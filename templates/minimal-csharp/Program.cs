using MinimalService;

if (args is ["--self-test"])
{
    SelfTests.Run();
    return 0;
}

var context = new RequestContext("demo-request");
var result = Dispatcher.Dispatch(new GreetingModule(), new Greet("World"), context);
switch (result)
{
    case Result<Greeting>.Success(var greeting):
        Console.WriteLine($"{greeting.Message} [{greeting.CorrelationId}]");
        return 0;
    case Result<Greeting>.Failure(var error):
        Console.Error.WriteLine($"{error.Code}: {error.Message}");
        return 1;
    default:
        throw new InvalidOperationException("Unknown result.");
}
