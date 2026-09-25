namespace MinimalService;

internal static class SelfTests
{
    public static void Run()
    {
        var module = new GreetingModule();
        var context = new RequestContext("test-request");
        var result = Dispatcher.Dispatch(module, new Greet(" Ada "), context);
        if (result is not Result<Greeting>.Success
            { Value: { Message: "Hello, Ada!", CorrelationId: "test-request" } })
        {
            throw new InvalidOperationException("Dispatch or context propagation failed.");
        }

        foreach (var name in new string?[] { null, "", " \t\n" })
        {
            if (Dispatcher.Dispatch(module, new Greet(name), context)
                is not Result<Greeting>.Failure
                { Error: { Code: "validation.name_required" } })
            {
                throw new InvalidOperationException("Blank name was not rejected.");
            }
        }

        Console.WriteLine("All self-tests passed.");
    }
}
