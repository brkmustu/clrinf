using System;

namespace EticaretApp.Application.Common.Functional;

public readonly struct Option<T>
{
    private readonly T? _value;
    public bool HasValue { get; }

    private Option(T value) { _value = value; HasValue = true; }

    public static Option<T> Some(T value)
    {
        if (value is null) throw new ArgumentNullException(nameof(value));
        return new Option<T>(value);
    }

    public static readonly Option<T> None = default;

    public TResult Match<TResult>(Func<T, TResult> onSome, Func<TResult> onNone) =>
        HasValue ? onSome(_value!) : onNone();
}
