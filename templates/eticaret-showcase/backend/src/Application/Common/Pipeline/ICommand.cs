namespace EticaretApp.Application.Common.Pipeline;

/// <summary>
/// Marker interface for command messages that return a response.
/// </summary>
/// <typeparam name="TResponse">The type of the response produced by the command handler.</typeparam>
public interface ICommand<out TResponse> { }

/// <summary>
/// Marker interface for command messages that do not return a meaningful response.
/// Internally maps to <see cref="Unit"/>.
/// </summary>
public interface ICommand : ICommand<Unit> { }
