namespace EcommerceMonolith.Common.Functional;

using System;
using System.Threading.Tasks;

public readonly record struct Result<T, TE>
{
    public bool IsSuccess { get; }
    public bool IsFailure => !IsSuccess;

    private readonly T? _value;
    private readonly TE? _error;

    private Result(T value)
    {
        IsSuccess = true;
        _value = value;
        _error = default;
    }

    private Result(TE error)
    {
        IsSuccess = false;
        _value = default;
        _error = error;
    }

    public T Value => IsSuccess ? _value! : throw new InvalidOperationException($"Cannot access Value of failed result: {_error}");
    public TE Error => IsFailure ? _error! : throw new InvalidOperationException("Cannot access Error of successful result.");

    public static Result<T, TE> Success(T value) => new(value);
    public static Result<T, TE> Failure(TE error) => new(error);

    public static implicit operator Result<T, TE>(T value) => new(value);
    public static implicit operator Result<T, TE>(TE error) => new(error);

    public TResult Match<TResult>(Func<T, TResult> onSuccess, Func<TE, TResult> onFailure) =>
        IsSuccess ? onSuccess(_value!) : onFailure(_error!);

    public async Task<TResult> MatchAsync<TResult>(Func<T, Task<TResult>> onSuccess, Func<TE, Task<TResult>> onFailure) =>
        IsSuccess ? await onSuccess(_value!) : await onFailure(_error!);

    public Result<TOut, TE> Map<TOut>(Func<T, TOut> mapper) =>
        IsSuccess ? Result<TOut, TE>.Success(mapper(_value!)) : Result<TOut, TE>.Failure(_error!);

    public Result<TOut, TE> Bind<TOut>(Func<T, Result<TOut, TE>> binder) =>
        IsSuccess ? binder(_value!) : Result<TOut, TE>.Failure(_error!);
}
