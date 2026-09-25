using System.Threading;
using System.Threading.Tasks;
using FluentValidation;
using EticaretApp.Application.Common.Attributes;
using EticaretApp.Application.Common.Functional;
using EticaretApp.Application.Common.Pipeline;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Common.Exceptions;
using EticaretApp.Domain.Entities;
using static EticaretApp.Application.Moduller.Kampanyalar.KampanyalarOperationClaims;

namespace EticaretApp.Application.Moduller.Kampanyalar;

#region Responses

public sealed record CreatedKampanyaResponse
{
    public int Id { get; init; }
    public string Kod { get; init; }
    public string Baslik { get; init; }
    public decimal Deger { get; init; }
    public bool Aktif { get; init; }
}

public sealed record UpdatedKampanyaResponse
{
    public int Id { get; init; }
    public string Kod { get; init; }
    public string Baslik { get; init; }
    public decimal Deger { get; init; }
    public bool Aktif { get; init; }
}

public sealed record DeletedKampanyaResponse
{
    public int Id { get; init; }
}

#endregion

#region Create Command & Handler

[RequireAuthorization(Admin, Write, KampanyalarOperationClaims.Create)]
[Logged]
[Transactional]
public sealed record CreateKampanyaCommand(
    string Kod, 
    string Baslik, 
    decimal Deger, 
    bool Aktif
) : ICommand<Result<CreatedKampanyaResponse, DomainError>>;

public class CreateKampanyaCommandValidator : AbstractValidator<CreateKampanyaCommand>
{
    public CreateKampanyaCommandValidator()
    {
    }
}

internal sealed class CreateKampanyaCommandHandler : ICommandHandler<CreateKampanyaCommand, Result<CreatedKampanyaResponse, DomainError>>
{
    private readonly IKampanyaRepository _kampanyaRepository;
    private readonly KampanyaBusinessRules _kampanyaBusinessRules;

    public CreateKampanyaCommandHandler(
        IKampanyaRepository kampanyaRepository,
        KampanyaBusinessRules kampanyaBusinessRules)
    {
        _kampanyaRepository = kampanyaRepository;
        _kampanyaBusinessRules = kampanyaBusinessRules;
    }

    public async ValueTask<Result<CreatedKampanyaResponse, DomainError>> HandleAsync(
        CreateKampanyaCommand request,
        CancellationToken cancellationToken)
    {
        Kampanya entity = request.ToEntity();
        await _kampanyaRepository.AddAsync(entity);
        return Result<CreatedKampanyaResponse, DomainError>.Success(entity.ToCreatedResponse());
    }
}

#endregion

#region Update Command & Handler

[RequireAuthorization(Admin, Write, KampanyalarOperationClaims.Update)]
[Logged]
[Transactional]
public sealed record UpdateKampanyaCommand(
    int Id,
    string Kod, 
    string Baslik, 
    decimal Deger, 
    bool Aktif
) : ICommand<Result<UpdatedKampanyaResponse, DomainError>>;

public class UpdateKampanyaCommandValidator : AbstractValidator<UpdateKampanyaCommand>
{
    public UpdateKampanyaCommandValidator()
    {
        RuleFor(c => c.Id).NotEmpty();
    }
}

internal sealed class UpdateKampanyaCommandHandler : ICommandHandler<UpdateKampanyaCommand, Result<UpdatedKampanyaResponse, DomainError>>
{
    private readonly IKampanyaRepository _kampanyaRepository;
    private readonly KampanyaBusinessRules _kampanyaBusinessRules;

    public UpdateKampanyaCommandHandler(
        IKampanyaRepository kampanyaRepository,
        KampanyaBusinessRules kampanyaBusinessRules)
    {
        _kampanyaRepository = kampanyaRepository;
        _kampanyaBusinessRules = kampanyaBusinessRules;
    }

    public async ValueTask<Result<UpdatedKampanyaResponse, DomainError>> HandleAsync(
        UpdateKampanyaCommand request,
        CancellationToken cancellationToken)
    {
        Kampanya? entity = await _kampanyaRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            cancellationToken: cancellationToken
        );
        await _kampanyaBusinessRules.KampanyaShouldExistWhenSelected(entity);

        entity = request.ToEntity();
        await _kampanyaRepository.UpdateAsync(entity);
        return Result<UpdatedKampanyaResponse, DomainError>.Success(entity.ToUpdatedResponse());
    }
}

#endregion

#region Delete Command & Handler

[RequireAuthorization(Admin, Write, KampanyalarOperationClaims.Delete)]
[Logged]
[Transactional]
public sealed record DeleteKampanyaCommand(int Id) : ICommand<Result<DeletedKampanyaResponse, DomainError>>;

public class DeleteKampanyaCommandValidator : AbstractValidator<DeleteKampanyaCommand>
{
    public DeleteKampanyaCommandValidator()
    {
        RuleFor(c => c.Id).NotEmpty();
    }
}

internal sealed class DeleteKampanyaCommandHandler : ICommandHandler<DeleteKampanyaCommand, Result<DeletedKampanyaResponse, DomainError>>
{
    private readonly IKampanyaRepository _kampanyaRepository;
    private readonly KampanyaBusinessRules _kampanyaBusinessRules;

    public DeleteKampanyaCommandHandler(
        IKampanyaRepository kampanyaRepository,
        KampanyaBusinessRules kampanyaBusinessRules)
    {
        _kampanyaRepository = kampanyaRepository;
        _kampanyaBusinessRules = kampanyaBusinessRules;
    }

    public async ValueTask<Result<DeletedKampanyaResponse, DomainError>> HandleAsync(
        DeleteKampanyaCommand request,
        CancellationToken cancellationToken)
    {
        Kampanya? entity = await _kampanyaRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            cancellationToken: cancellationToken
        );
        await _kampanyaBusinessRules.KampanyaShouldExistWhenSelected(entity);

        await _kampanyaRepository.DeleteAsync(entity!);
        return Result<DeletedKampanyaResponse, DomainError>.Success(entity!.ToDeletedResponse());
    }
}

#endregion
