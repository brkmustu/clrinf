namespace EticaretApp.Application.Common.Pipeline;

/// <summary>
/// Marker interface for query messages.
/// Queries are read-only operations that return data without side effects.
/// </summary>
/// <typeparam name="TResponse">The type of the data returned by the query handler.</typeparam>
public interface IQuery<out TResponse> { }
