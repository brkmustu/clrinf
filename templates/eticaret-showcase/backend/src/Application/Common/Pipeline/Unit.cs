using System.Threading.Tasks;

namespace EticaretApp.Application.Common.Pipeline;

/// <summary>
/// Represents the absence of a meaningful return value.
/// Used as <c>TResponse</c> for commands that do not produce a result.
/// </summary>
public readonly record struct Unit
{
    /// <summary>Singleton value.</summary>
    public static readonly Unit Value = new();

    /// <summary>A completed task returning <see cref="Value"/>.</summary>
    public static readonly Task<Unit> Task = System.Threading.Tasks.Task.FromResult(Value);
}
