using System.Threading;
using System.Threading.Tasks;
using FluentValidation;
using EticaretApp.Application.Common.Attributes;
using EticaretApp.Application.Common.Functional;
using EticaretApp.Application.Common.Pipeline;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Common.Exceptions;
using EticaretApp.Domain.Entities;
using static EticaretApp.Application.Features.Kuponlar.KuponlarOperationClaims;

namespace EticaretApp.Application.Features.Kuponlar;

#region Responses

public sealed record CreatedKuponResponse
{
    public int Id { get; init; }
}

public sealed record UpdatedKuponResponse
{
    public int Id { get; init; }
}

public sealed record DeletedKuponResponse
{
    public int Id { get; init; }
}

#endregion

#region Create Command & Handler

[RequireAuthorization(Admin, Write, KuponlarOperationClaims.Create)]
[Logged]
[Transactional]
public sealed record CreateKuponCommand(
) : ICommand<Result<CreatedKuponResponse, DomainError>>;

public class CreateKuponCommandValidator : AbstractValidator<CreateKuponCommand>
{
    public CreateKuponCommandValidator()
    {
    }
}

internal sealed class CreateKuponCommandHandler : ICommandHandler<CreateKuponCommand, Result<CreatedKuponResponse, DomainError>>
{
    private readonly IKuponRepository _kuponRepository;
    private readonly KuponBusinessRules _kuponBusinessRules;

    public CreateKuponCommandHandler(
        IKuponRepository kuponRepository,
        KuponBusinessRules kuponBusinessRules)
    {
        _kuponRepository = kuponRepository;
        _kuponBusinessRules = kuponBusinessRules;
    }

    public async ValueTask<Result<CreatedKuponResponse, DomainError>> HandleAsync(
        CreateKuponCommand request,
        CancellationToken cancellationToken)
    {
        Kupon entity = request.ToEntity();
        await _kuponRepository.AddAsync(entity);
        return Result<CreatedKuponResponse, DomainError>.Success(entity.ToCreatedResponse());
    }
}

#endregion

#region Update Command & Handler

[RequireAuthorization(Admin, Write, KuponlarOperationClaims.Update)]
[Logged]
[Transactional]
public sealed record UpdateKuponCommand(
    int Id) : ICommand<Result<UpdatedKuponResponse, DomainError>>;

public class UpdateKuponCommandValidator : AbstractValidator<UpdateKuponCommand>
{
    public UpdateKuponCommandValidator()
    {
        RuleFor(c => c.Id).NotEmpty();
    }
}

internal sealed class UpdateKuponCommandHandler : ICommandHandler<UpdateKuponCommand, Result<UpdatedKuponResponse, DomainError>>
{
    private readonly IKuponRepository _kuponRepository;
    private readonly KuponBusinessRules _kuponBusinessRules;

    public UpdateKuponCommandHandler(
        IKuponRepository kuponRepository,
        KuponBusinessRules kuponBusinessRules)
    {
        _kuponRepository = kuponRepository;
        _kuponBusinessRules = kuponBusinessRules;
    }

    public async ValueTask<Result<UpdatedKuponResponse, DomainError>> HandleAsync(
        UpdateKuponCommand request,
        CancellationToken cancellationToken)
    {
        Kupon? entity = await _kuponRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            cancellationToken: cancellationToken
        );
        await _kuponBusinessRules.KuponShouldExistWhenSelected(entity);

        entity = request.ToEntity();
        await _kuponRepository.UpdateAsync(entity);
        return Result<UpdatedKuponResponse, DomainError>.Success(entity.ToUpdatedResponse());
    }
}

#endregion

#region Delete Command & Handler

[RequireAuthorization(Admin, Write, KuponlarOperationClaims.Delete)]
[Logged]
[Transactional]
public sealed record DeleteKuponCommand(int Id) : ICommand<Result<DeletedKuponResponse, DomainError>>;

public class DeleteKuponCommandValidator : AbstractValidator<DeleteKuponCommand>
{
    public DeleteKuponCommandValidator()
    {
        RuleFor(c => c.Id).NotEmpty();
    }
}

internal sealed class DeleteKuponCommandHandler : ICommandHandler<DeleteKuponCommand, Result<DeletedKuponResponse, DomainError>>
{
    private readonly IKuponRepository _kuponRepository;
    private readonly KuponBusinessRules _kuponBusinessRules;

    public DeleteKuponCommandHandler(
        IKuponRepository kuponRepository,
        KuponBusinessRules kuponBusinessRules)
    {
        _kuponRepository = kuponRepository;
        _kuponBusinessRules = kuponBusinessRules;
    }

    public async ValueTask<Result<DeletedKuponResponse, DomainError>> HandleAsync(
        DeleteKuponCommand request,
        CancellationToken cancellationToken)
    {
        Kupon? entity = await _kuponRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            cancellationToken: cancellationToken
        );
        await _kuponBusinessRules.KuponShouldExistWhenSelected(entity);

        await _kuponRepository.DeleteAsync(entity!);
        return Result<DeletedKuponResponse, DomainError>.Success(entity!.ToDeletedResponse());
    }
}

#endregion
