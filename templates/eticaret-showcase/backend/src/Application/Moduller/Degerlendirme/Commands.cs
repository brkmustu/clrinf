using System.Threading;
using System.Threading.Tasks;
using FluentValidation;
using EticaretApp.Application.Common.Attributes;
using EticaretApp.Application.Common.Functional;
using EticaretApp.Application.Common.Pipeline;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Common.Exceptions;
using EticaretApp.Domain.Entities;
using static EticaretApp.Application.Features.Degerlendirmeler.DegerlendirmelerOperationClaims;

namespace EticaretApp.Application.Features.Degerlendirmeler;

#region Responses

public sealed record CreatedDegerlendirmeResponse
{
    public int Id { get; init; }
}

public sealed record UpdatedDegerlendirmeResponse
{
    public int Id { get; init; }
}

public sealed record DeletedDegerlendirmeResponse
{
    public int Id { get; init; }
}

#endregion

#region Create Command & Handler

[RequireAuthorization(Admin, Write, DegerlendirmelerOperationClaims.Create)]
[Logged]
[Transactional]
public sealed record CreateDegerlendirmeCommand(
) : ICommand<Result<CreatedDegerlendirmeResponse, DomainError>>;

public class CreateDegerlendirmeCommandValidator : AbstractValidator<CreateDegerlendirmeCommand>
{
    public CreateDegerlendirmeCommandValidator()
    {
    }
}

internal sealed class CreateDegerlendirmeCommandHandler : ICommandHandler<CreateDegerlendirmeCommand, Result<CreatedDegerlendirmeResponse, DomainError>>
{
    private readonly IDegerlendirmeRepository _degerlendirmeRepository;
    private readonly DegerlendirmeBusinessRules _degerlendirmeBusinessRules;

    public CreateDegerlendirmeCommandHandler(
        IDegerlendirmeRepository degerlendirmeRepository,
        DegerlendirmeBusinessRules degerlendirmeBusinessRules)
    {
        _degerlendirmeRepository = degerlendirmeRepository;
        _degerlendirmeBusinessRules = degerlendirmeBusinessRules;
    }

    public async ValueTask<Result<CreatedDegerlendirmeResponse, DomainError>> HandleAsync(
        CreateDegerlendirmeCommand request,
        CancellationToken cancellationToken)
    {
        Degerlendirme entity = request.ToEntity();
        await _degerlendirmeRepository.AddAsync(entity);
        return Result<CreatedDegerlendirmeResponse, DomainError>.Success(entity.ToCreatedResponse());
    }
}

#endregion

#region Update Command & Handler

[RequireAuthorization(Admin, Write, DegerlendirmelerOperationClaims.Update)]
[Logged]
[Transactional]
public sealed record UpdateDegerlendirmeCommand(
    int Id) : ICommand<Result<UpdatedDegerlendirmeResponse, DomainError>>;

public class UpdateDegerlendirmeCommandValidator : AbstractValidator<UpdateDegerlendirmeCommand>
{
    public UpdateDegerlendirmeCommandValidator()
    {
        RuleFor(c => c.Id).NotEmpty();
    }
}

internal sealed class UpdateDegerlendirmeCommandHandler : ICommandHandler<UpdateDegerlendirmeCommand, Result<UpdatedDegerlendirmeResponse, DomainError>>
{
    private readonly IDegerlendirmeRepository _degerlendirmeRepository;
    private readonly DegerlendirmeBusinessRules _degerlendirmeBusinessRules;

    public UpdateDegerlendirmeCommandHandler(
        IDegerlendirmeRepository degerlendirmeRepository,
        DegerlendirmeBusinessRules degerlendirmeBusinessRules)
    {
        _degerlendirmeRepository = degerlendirmeRepository;
        _degerlendirmeBusinessRules = degerlendirmeBusinessRules;
    }

    public async ValueTask<Result<UpdatedDegerlendirmeResponse, DomainError>> HandleAsync(
        UpdateDegerlendirmeCommand request,
        CancellationToken cancellationToken)
    {
        Degerlendirme? entity = await _degerlendirmeRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            cancellationToken: cancellationToken
        );
        await _degerlendirmeBusinessRules.DegerlendirmeShouldExistWhenSelected(entity);

        entity = request.ToEntity();
        await _degerlendirmeRepository.UpdateAsync(entity);
        return Result<UpdatedDegerlendirmeResponse, DomainError>.Success(entity.ToUpdatedResponse());
    }
}

#endregion

#region Delete Command & Handler

[RequireAuthorization(Admin, Write, DegerlendirmelerOperationClaims.Delete)]
[Logged]
[Transactional]
public sealed record DeleteDegerlendirmeCommand(int Id) : ICommand<Result<DeletedDegerlendirmeResponse, DomainError>>;

public class DeleteDegerlendirmeCommandValidator : AbstractValidator<DeleteDegerlendirmeCommand>
{
    public DeleteDegerlendirmeCommandValidator()
    {
        RuleFor(c => c.Id).NotEmpty();
    }
}

internal sealed class DeleteDegerlendirmeCommandHandler : ICommandHandler<DeleteDegerlendirmeCommand, Result<DeletedDegerlendirmeResponse, DomainError>>
{
    private readonly IDegerlendirmeRepository _degerlendirmeRepository;
    private readonly DegerlendirmeBusinessRules _degerlendirmeBusinessRules;

    public DeleteDegerlendirmeCommandHandler(
        IDegerlendirmeRepository degerlendirmeRepository,
        DegerlendirmeBusinessRules degerlendirmeBusinessRules)
    {
        _degerlendirmeRepository = degerlendirmeRepository;
        _degerlendirmeBusinessRules = degerlendirmeBusinessRules;
    }

    public async ValueTask<Result<DeletedDegerlendirmeResponse, DomainError>> HandleAsync(
        DeleteDegerlendirmeCommand request,
        CancellationToken cancellationToken)
    {
        Degerlendirme? entity = await _degerlendirmeRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            cancellationToken: cancellationToken
        );
        await _degerlendirmeBusinessRules.DegerlendirmeShouldExistWhenSelected(entity);

        await _degerlendirmeRepository.DeleteAsync(entity!);
        return Result<DeletedDegerlendirmeResponse, DomainError>.Success(entity!.ToDeletedResponse());
    }
}

#endregion
