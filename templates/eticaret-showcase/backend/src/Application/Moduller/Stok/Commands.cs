using System.Threading;
using System.Threading.Tasks;
using FluentValidation;
using EticaretApp.Application.Common.Attributes;
using EticaretApp.Application.Common.Functional;
using EticaretApp.Application.Common.Pipeline;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Common.Exceptions;
using EticaretApp.Domain.Entities;
using static EticaretApp.Application.Moduller.Stoklar.StoklarOperationClaims;

namespace EticaretApp.Application.Moduller.Stoklar;

#region Responses

public sealed record CreatedStokResponse
{
    public int Id { get; init; }
    public string Sku { get; init; }
    public int Miktar { get; init; }
    public int RezerveMiktar { get; init; }
}

public sealed record UpdatedStokResponse
{
    public int Id { get; init; }
    public string Sku { get; init; }
    public int Miktar { get; init; }
    public int RezerveMiktar { get; init; }
}

public sealed record DeletedStokResponse
{
    public int Id { get; init; }
}

#endregion

#region Create Command & Handler

[RequireAuthorization(Admin, Write, StoklarOperationClaims.Create)]
[Logged]
[Transactional]
public sealed record CreateStokCommand(
    string Sku, 
    int Miktar, 
    int RezerveMiktar
) : ICommand<Result<CreatedStokResponse, DomainError>>;

public class CreateStokCommandValidator : AbstractValidator<CreateStokCommand>
{
    public CreateStokCommandValidator()
    {
    }
}

internal sealed class CreateStokCommandHandler : ICommandHandler<CreateStokCommand, Result<CreatedStokResponse, DomainError>>
{
    private readonly IStokRepository _stokRepository;
    private readonly StokBusinessRules _stokBusinessRules;

    public CreateStokCommandHandler(
        IStokRepository stokRepository,
        StokBusinessRules stokBusinessRules)
    {
        _stokRepository = stokRepository;
        _stokBusinessRules = stokBusinessRules;
    }

    public async ValueTask<Result<CreatedStokResponse, DomainError>> HandleAsync(
        CreateStokCommand request,
        CancellationToken cancellationToken)
    {
        Stok entity = request.ToEntity();
        await _stokRepository.AddAsync(entity);
        return Result<CreatedStokResponse, DomainError>.Success(entity.ToCreatedResponse());
    }
}

#endregion

#region Update Command & Handler

[RequireAuthorization(Admin, Write, StoklarOperationClaims.Update)]
[Logged]
[Transactional]
public sealed record UpdateStokCommand(
    int Id,
    string Sku, 
    int Miktar, 
    int RezerveMiktar
) : ICommand<Result<UpdatedStokResponse, DomainError>>;

public class UpdateStokCommandValidator : AbstractValidator<UpdateStokCommand>
{
    public UpdateStokCommandValidator()
    {
        RuleFor(c => c.Id).NotEmpty();
    }
}

internal sealed class UpdateStokCommandHandler : ICommandHandler<UpdateStokCommand, Result<UpdatedStokResponse, DomainError>>
{
    private readonly IStokRepository _stokRepository;
    private readonly StokBusinessRules _stokBusinessRules;

    public UpdateStokCommandHandler(
        IStokRepository stokRepository,
        StokBusinessRules stokBusinessRules)
    {
        _stokRepository = stokRepository;
        _stokBusinessRules = stokBusinessRules;
    }

    public async ValueTask<Result<UpdatedStokResponse, DomainError>> HandleAsync(
        UpdateStokCommand request,
        CancellationToken cancellationToken)
    {
        Stok? entity = await _stokRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            cancellationToken: cancellationToken
        );
        await _stokBusinessRules.StokShouldExistWhenSelected(entity);

        entity = request.ToEntity();
        await _stokRepository.UpdateAsync(entity);
        return Result<UpdatedStokResponse, DomainError>.Success(entity.ToUpdatedResponse());
    }
}

#endregion

#region Delete Command & Handler

[RequireAuthorization(Admin, Write, StoklarOperationClaims.Delete)]
[Logged]
[Transactional]
public sealed record DeleteStokCommand(int Id) : ICommand<Result<DeletedStokResponse, DomainError>>;

public class DeleteStokCommandValidator : AbstractValidator<DeleteStokCommand>
{
    public DeleteStokCommandValidator()
    {
        RuleFor(c => c.Id).NotEmpty();
    }
}

internal sealed class DeleteStokCommandHandler : ICommandHandler<DeleteStokCommand, Result<DeletedStokResponse, DomainError>>
{
    private readonly IStokRepository _stokRepository;
    private readonly StokBusinessRules _stokBusinessRules;

    public DeleteStokCommandHandler(
        IStokRepository stokRepository,
        StokBusinessRules stokBusinessRules)
    {
        _stokRepository = stokRepository;
        _stokBusinessRules = stokBusinessRules;
    }

    public async ValueTask<Result<DeletedStokResponse, DomainError>> HandleAsync(
        DeleteStokCommand request,
        CancellationToken cancellationToken)
    {
        Stok? entity = await _stokRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            cancellationToken: cancellationToken
        );
        await _stokBusinessRules.StokShouldExistWhenSelected(entity);

        await _stokRepository.DeleteAsync(entity!);
        return Result<DeletedStokResponse, DomainError>.Success(entity!.ToDeletedResponse());
    }
}

#endregion
