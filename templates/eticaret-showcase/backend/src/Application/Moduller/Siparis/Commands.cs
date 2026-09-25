using System.Threading;
using System.Threading.Tasks;
using FluentValidation;
using EticaretApp.Application.Common.Attributes;
using EticaretApp.Application.Common.Functional;
using EticaretApp.Application.Common.Pipeline;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Common.Exceptions;
using EticaretApp.Domain.Entities;
using static EticaretApp.Application.Moduller.Siparisler.SiparislerOperationClaims;

namespace EticaretApp.Application.Moduller.Siparisler;

#region Responses

public sealed record CreatedSiparisResponse
{
    public int Id { get; init; }
    public string SiparisNo { get; init; }
    public string KullaniciId { get; init; }
    public decimal ToplamTutar { get; init; }
    public string Durum { get; init; }
    public string? KargoTakipNo { get; init; }
}

public sealed record UpdatedSiparisResponse
{
    public int Id { get; init; }
    public string SiparisNo { get; init; }
    public string KullaniciId { get; init; }
    public decimal ToplamTutar { get; init; }
    public string Durum { get; init; }
    public string? KargoTakipNo { get; init; }
}

public sealed record DeletedSiparisResponse
{
    public int Id { get; init; }
}

#endregion

#region Create Command & Handler

[RequireAuthorization(Admin, Write, SiparislerOperationClaims.Create)]
[Logged]
[Transactional]
public sealed record CreateSiparisCommand(
    string SiparisNo, 
    string KullaniciId, 
    decimal ToplamTutar, 
    string Durum, 
    string? KargoTakipNo
) : ICommand<Result<CreatedSiparisResponse, DomainError>>;

public class CreateSiparisCommandValidator : AbstractValidator<CreateSiparisCommand>
{
    public CreateSiparisCommandValidator()
    {
    }
}

internal sealed class CreateSiparisCommandHandler : ICommandHandler<CreateSiparisCommand, Result<CreatedSiparisResponse, DomainError>>
{
    private readonly ISiparisRepository _siparisRepository;
    private readonly SiparisBusinessRules _siparisBusinessRules;

    public CreateSiparisCommandHandler(
        ISiparisRepository siparisRepository,
        SiparisBusinessRules siparisBusinessRules)
    {
        _siparisRepository = siparisRepository;
        _siparisBusinessRules = siparisBusinessRules;
    }

    public async ValueTask<Result<CreatedSiparisResponse, DomainError>> HandleAsync(
        CreateSiparisCommand request,
        CancellationToken cancellationToken)
    {
        Siparis entity = request.ToEntity();
        await _siparisRepository.AddAsync(entity);
        return Result<CreatedSiparisResponse, DomainError>.Success(entity.ToCreatedResponse());
    }
}

#endregion

#region Update Command & Handler

[RequireAuthorization(Admin, Write, SiparislerOperationClaims.Update)]
[Logged]
[Transactional]
public sealed record UpdateSiparisCommand(
    int Id,
    string SiparisNo, 
    string KullaniciId, 
    decimal ToplamTutar, 
    string Durum, 
    string? KargoTakipNo
) : ICommand<Result<UpdatedSiparisResponse, DomainError>>;

public class UpdateSiparisCommandValidator : AbstractValidator<UpdateSiparisCommand>
{
    public UpdateSiparisCommandValidator()
    {
        RuleFor(c => c.Id).NotEmpty();
    }
}

internal sealed class UpdateSiparisCommandHandler : ICommandHandler<UpdateSiparisCommand, Result<UpdatedSiparisResponse, DomainError>>
{
    private readonly ISiparisRepository _siparisRepository;
    private readonly SiparisBusinessRules _siparisBusinessRules;

    public UpdateSiparisCommandHandler(
        ISiparisRepository siparisRepository,
        SiparisBusinessRules siparisBusinessRules)
    {
        _siparisRepository = siparisRepository;
        _siparisBusinessRules = siparisBusinessRules;
    }

    public async ValueTask<Result<UpdatedSiparisResponse, DomainError>> HandleAsync(
        UpdateSiparisCommand request,
        CancellationToken cancellationToken)
    {
        Siparis? entity = await _siparisRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            cancellationToken: cancellationToken
        );
        await _siparisBusinessRules.SiparisShouldExistWhenSelected(entity);

        entity = request.ToEntity();
        await _siparisRepository.UpdateAsync(entity);
        return Result<UpdatedSiparisResponse, DomainError>.Success(entity.ToUpdatedResponse());
    }
}

#endregion

#region Delete Command & Handler

[RequireAuthorization(Admin, Write, SiparislerOperationClaims.Delete)]
[Logged]
[Transactional]
public sealed record DeleteSiparisCommand(int Id) : ICommand<Result<DeletedSiparisResponse, DomainError>>;

public class DeleteSiparisCommandValidator : AbstractValidator<DeleteSiparisCommand>
{
    public DeleteSiparisCommandValidator()
    {
        RuleFor(c => c.Id).NotEmpty();
    }
}

internal sealed class DeleteSiparisCommandHandler : ICommandHandler<DeleteSiparisCommand, Result<DeletedSiparisResponse, DomainError>>
{
    private readonly ISiparisRepository _siparisRepository;
    private readonly SiparisBusinessRules _siparisBusinessRules;

    public DeleteSiparisCommandHandler(
        ISiparisRepository siparisRepository,
        SiparisBusinessRules siparisBusinessRules)
    {
        _siparisRepository = siparisRepository;
        _siparisBusinessRules = siparisBusinessRules;
    }

    public async ValueTask<Result<DeletedSiparisResponse, DomainError>> HandleAsync(
        DeleteSiparisCommand request,
        CancellationToken cancellationToken)
    {
        Siparis? entity = await _siparisRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            cancellationToken: cancellationToken
        );
        await _siparisBusinessRules.SiparisShouldExistWhenSelected(entity);

        await _siparisRepository.DeleteAsync(entity!);
        return Result<DeletedSiparisResponse, DomainError>.Success(entity!.ToDeletedResponse());
    }
}

#endregion
