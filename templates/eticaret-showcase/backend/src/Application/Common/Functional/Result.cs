using System;
using System.Threading.Tasks;

namespace EticaretApp.Application.Common.Functional;

public readonly struct Result<TValue, TError>
{
    private readonly TValue? _value;
    private readonly TError? _error;
    public bool IsSuccess { get; }
    public bool IsFailure => !IsSuccess;

    private Result(TValue value)
    {
        _value = value;
        _error = default;
        IsSuccess = true;
    }

    private Result(TError error)
    {
        _value = default;
        _error = error;
        IsSuccess = false;
    }

    public static Result<TValue, TError> Success(TValue value) => new(value);
    public static Result<TValue, TError> Failure(TError error) => new(error);

    public TValue Value => IsSuccess
        ? _value!
        : throw new InvalidOperationException("Cannot access Value of a failed Result.");

    public TError Error => IsFailure
        ? _error!
        : throw new InvalidOperationException("Cannot access Error of a successful Result.");

    public TResult Match<TResult>(
        Func<TValue, TResult> onSuccess,
        Func<TError, TResult> onFailure) =>
        IsSuccess ? onSuccess(_value!) : onFailure(_error!);

    public Result<TOut, TError> Map<TOut>(Func<TValue, TOut> map) =>
        IsSuccess ? Result<TOut, TError>.Success(map(_value!))
                  : Result<TOut, TError>.Failure(_error!);

    public Result<TOut, TError> Bind<TOut>(Func<TValue, Result<TOut, TError>> bind) =>
        IsSuccess ? bind(_value!) : Result<TOut, TError>.Failure(_error!);
}

public static class ResultExtensions
{
    public static async Task<Result<TOut, TError>> BindAsync<TIn, TOut, TError>(
        this Result<TIn, TError> result,
        Func<TIn, Task<Result<TOut, TError>>> bind) =>
        result.IsSuccess ? await bind(result.Value) : Result<TOut, TError>.Failure(result.Error);
}
