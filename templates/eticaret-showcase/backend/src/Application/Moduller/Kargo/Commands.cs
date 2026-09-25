using System.Threading;
using System.Threading.Tasks;
using FluentValidation;
using EticaretApp.Application.Common.Attributes;
using EticaretApp.Application.Common.Functional;
using EticaretApp.Application.Common.Pipeline;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Common.Exceptions;
using EticaretApp.Domain.Entities;
using static EticaretApp.Application.Features.Kargolar.KargolarOperationClaims;

namespace EticaretApp.Application.Features.Kargolar;

#region Responses

public sealed record CreatedKargoResponse
{
    public int Id { get; init; }
}

public sealed record UpdatedKargoResponse
{
    public int Id { get; init; }
}

public sealed record DeletedKargoResponse
{
    public int Id { get; init; }
}

#endregion

#region Create Command & Handler

[RequireAuthorization(Admin, Write, KargolarOperationClaims.Create)]
[Logged]
[Transactional]
public sealed record CreateKargoCommand(
) : ICommand<Result<CreatedKargoResponse, DomainError>>;

public class CreateKargoCommandValidator : AbstractValidator<CreateKargoCommand>
{
    public CreateKargoCommandValidator()
    {
    }
}

internal sealed class CreateKargoCommandHandler : ICommandHandler<CreateKargoCommand, Result<CreatedKargoResponse, DomainError>>
{
    private readonly IKargoRepository _kargoRepository;
    private readonly KargoBusinessRules _kargoBusinessRules;

    public CreateKargoCommandHandler(
        IKargoRepository kargoRepository,
        KargoBusinessRules kargoBusinessRules)
    {
        _kargoRepository = kargoRepository;
        _kargoBusinessRules = kargoBusinessRules;
    }

    public async ValueTask<Result<CreatedKargoResponse, DomainError>> HandleAsync(
        CreateKargoCommand request,
        CancellationToken cancellationToken)
    {
        Kargo entity = request.ToEntity();
        await _kargoRepository.AddAsync(entity);
        return Result<CreatedKargoResponse, DomainError>.Success(entity.ToCreatedResponse());
    }
}

#endregion

#region Update Command & Handler

[RequireAuthorization(Admin, Write, KargolarOperationClaims.Update)]
[Logged]
[Transactional]
public sealed record UpdateKargoCommand(
    int Id) : ICommand<Result<UpdatedKargoResponse, DomainError>>;

public class UpdateKargoCommandValidator : AbstractValidator<UpdateKargoCommand>
{
    public UpdateKargoCommandValidator()
    {
        RuleFor(c => c.Id).NotEmpty();
    }
}

internal sealed class UpdateKargoCommandHandler : ICommandHandler<UpdateKargoCommand, Result<UpdatedKargoResponse, DomainError>>
{
    private readonly IKargoRepository _kargoRepository;
    private readonly KargoBusinessRules _kargoBusinessRules;

    public UpdateKargoCommandHandler(
        IKargoRepository kargoRepository,
        KargoBusinessRules kargoBusinessRules)
    {
        _kargoRepository = kargoRepository;
        _kargoBusinessRules = kargoBusinessRules;
    }

    public async ValueTask<Result<UpdatedKargoResponse, DomainError>> HandleAsync(
        UpdateKargoCommand request,
        CancellationToken cancellationToken)
    {
        Kargo? entity = await _kargoRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            cancellationToken: cancellationToken
        );
        await _kargoBusinessRules.KargoShouldExistWhenSelected(entity);

        entity = request.ToEntity();
        await _kargoRepository.UpdateAsync(entity);
        return Result<UpdatedKargoResponse, DomainError>.Success(entity.ToUpdatedResponse());
    }
}

#endregion

#region Delete Command & Handler

[RequireAuthorization(Admin, Write, KargolarOperationClaims.Delete)]
[Logged]
[Transactional]
public sealed record DeleteKargoCommand(int Id) : ICommand<Result<DeletedKargoResponse, DomainError>>;

public class DeleteKargoCommandValidator : AbstractValidator<DeleteKargoCommand>
{
    public DeleteKargoCommandValidator()
    {
        RuleFor(c => c.Id).NotEmpty();
    }
}

internal sealed class DeleteKargoCommandHandler : ICommandHandler<DeleteKargoCommand, Result<DeletedKargoResponse, DomainError>>
{
    private readonly IKargoRepository _kargoRepository;
    private readonly KargoBusinessRules _kargoBusinessRules;

    public DeleteKargoCommandHandler(
        IKargoRepository kargoRepository,
        KargoBusinessRules kargoBusinessRules)
    {
        _kargoRepository = kargoRepository;
        _kargoBusinessRules = kargoBusinessRules;
    }

    public async ValueTask<Result<DeletedKargoResponse, DomainError>> HandleAsync(
        DeleteKargoCommand request,
        CancellationToken cancellationToken)
    {
        Kargo? entity = await _kargoRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            cancellationToken: cancellationToken
        );
        await _kargoBusinessRules.KargoShouldExistWhenSelected(entity);

        await _kargoRepository.DeleteAsync(entity!);
        return Result<DeletedKargoResponse, DomainError>.Success(entity!.ToDeletedResponse());
    }
}

#endregion
