using System.Threading;
using System.Threading.Tasks;
using FluentValidation;
using EticaretApp.Application.Common.Attributes;
using EticaretApp.Application.Common.Functional;
using EticaretApp.Application.Common.Pipeline;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Common.Exceptions;
using EticaretApp.Domain.Entities;
using static EticaretApp.Application.Features.Odemeler.OdemelerOperationClaims;

namespace EticaretApp.Application.Features.Odemeler;

#region Responses

public sealed record CreatedOdemeResponse
{
    public int Id { get; init; }
}

public sealed record UpdatedOdemeResponse
{
    public int Id { get; init; }
}

public sealed record DeletedOdemeResponse
{
    public int Id { get; init; }
}

#endregion

#region Create Command & Handler

[RequireAuthorization(Admin, Write, OdemelerOperationClaims.Create)]
[Logged]
[Transactional]
public sealed record CreateOdemeCommand(
) : ICommand<Result<CreatedOdemeResponse, DomainError>>;

public class CreateOdemeCommandValidator : AbstractValidator<CreateOdemeCommand>
{
    public CreateOdemeCommandValidator()
    {
    }
}

internal sealed class CreateOdemeCommandHandler : ICommandHandler<CreateOdemeCommand, Result<CreatedOdemeResponse, DomainError>>
{
    private readonly IOdemeRepository _odemeRepository;
    private readonly OdemeBusinessRules _odemeBusinessRules;

    public CreateOdemeCommandHandler(
        IOdemeRepository odemeRepository,
        OdemeBusinessRules odemeBusinessRules)
    {
        _odemeRepository = odemeRepository;
        _odemeBusinessRules = odemeBusinessRules;
    }

    public async ValueTask<Result<CreatedOdemeResponse, DomainError>> HandleAsync(
        CreateOdemeCommand request,
        CancellationToken cancellationToken)
    {
        Odeme entity = request.ToEntity();
        await _odemeRepository.AddAsync(entity);
        return Result<CreatedOdemeResponse, DomainError>.Success(entity.ToCreatedResponse());
    }
}

#endregion

#region Update Command & Handler

[RequireAuthorization(Admin, Write, OdemelerOperationClaims.Update)]
[Logged]
[Transactional]
public sealed record UpdateOdemeCommand(
    int Id) : ICommand<Result<UpdatedOdemeResponse, DomainError>>;

public class UpdateOdemeCommandValidator : AbstractValidator<UpdateOdemeCommand>
{
    public UpdateOdemeCommandValidator()
    {
        RuleFor(c => c.Id).NotEmpty();
    }
}

internal sealed class UpdateOdemeCommandHandler : ICommandHandler<UpdateOdemeCommand, Result<UpdatedOdemeResponse, DomainError>>
{
    private readonly IOdemeRepository _odemeRepository;
    private readonly OdemeBusinessRules _odemeBusinessRules;

    public UpdateOdemeCommandHandler(
        IOdemeRepository odemeRepository,
        OdemeBusinessRules odemeBusinessRules)
    {
        _odemeRepository = odemeRepository;
        _odemeBusinessRules = odemeBusinessRules;
    }

    public async ValueTask<Result<UpdatedOdemeResponse, DomainError>> HandleAsync(
        UpdateOdemeCommand request,
        CancellationToken cancellationToken)
    {
        Odeme? entity = await _odemeRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            cancellationToken: cancellationToken
        );
        await _odemeBusinessRules.OdemeShouldExistWhenSelected(entity);

        entity = request.ToEntity();
        await _odemeRepository.UpdateAsync(entity);
        return Result<UpdatedOdemeResponse, DomainError>.Success(entity.ToUpdatedResponse());
    }
}

#endregion

#region Delete Command & Handler

[RequireAuthorization(Admin, Write, OdemelerOperationClaims.Delete)]
[Logged]
[Transactional]
public sealed record DeleteOdemeCommand(int Id) : ICommand<Result<DeletedOdemeResponse, DomainError>>;

public class DeleteOdemeCommandValidator : AbstractValidator<DeleteOdemeCommand>
{
    public DeleteOdemeCommandValidator()
    {
        RuleFor(c => c.Id).NotEmpty();
    }
}

internal sealed class DeleteOdemeCommandHandler : ICommandHandler<DeleteOdemeCommand, Result<DeletedOdemeResponse, DomainError>>
{
    private readonly IOdemeRepository _odemeRepository;
    private readonly OdemeBusinessRules _odemeBusinessRules;

    public DeleteOdemeCommandHandler(
        IOdemeRepository odemeRepository,
        OdemeBusinessRules odemeBusinessRules)
    {
        _odemeRepository = odemeRepository;
        _odemeBusinessRules = odemeBusinessRules;
    }

    public async ValueTask<Result<DeletedOdemeResponse, DomainError>> HandleAsync(
        DeleteOdemeCommand request,
        CancellationToken cancellationToken)
    {
        Odeme? entity = await _odemeRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            cancellationToken: cancellationToken
        );
        await _odemeBusinessRules.OdemeShouldExistWhenSelected(entity);

        await _odemeRepository.DeleteAsync(entity!);
        return Result<DeletedOdemeResponse, DomainError>.Success(entity!.ToDeletedResponse());
    }
}

#endregion
