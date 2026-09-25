using System.Threading;
using System.Threading.Tasks;
using FluentValidation;
using EticaretApp.Application.Common.Attributes;
using EticaretApp.Application.Common.Functional;
using EticaretApp.Application.Common.Pipeline;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Common.Exceptions;
using EticaretApp.Domain.Entities;
using static EticaretApp.Application.Moduller.Sepetler.SepetlerOperationClaims;

namespace EticaretApp.Application.Moduller.Sepetler;

#region Responses

public sealed record CreatedSepetResponse
{
    public int Id { get; init; }
    public string KullaniciId { get; init; }
    public decimal ToplamTutar { get; init; }
    public decimal IndirimTutari { get; init; }
    public bool Kilitli { get; init; }
}

public sealed record UpdatedSepetResponse
{
    public int Id { get; init; }
    public string KullaniciId { get; init; }
    public decimal ToplamTutar { get; init; }
    public decimal IndirimTutari { get; init; }
    public bool Kilitli { get; init; }
}

public sealed record DeletedSepetResponse
{
    public int Id { get; init; }
}

#endregion

#region Create Command & Handler

[RequireAuthorization(Admin, Write, SepetlerOperationClaims.Create)]
[Logged]
[Transactional]
public sealed record CreateSepetCommand(
    string KullaniciId, 
    decimal ToplamTutar, 
    decimal IndirimTutari, 
    bool Kilitli
) : ICommand<Result<CreatedSepetResponse, DomainError>>;

public class CreateSepetCommandValidator : AbstractValidator<CreateSepetCommand>
{
    public CreateSepetCommandValidator()
    {
    }
}

internal sealed class CreateSepetCommandHandler : ICommandHandler<CreateSepetCommand, Result<CreatedSepetResponse, DomainError>>
{
    private readonly ISepetRepository _sepetRepository;
    private readonly SepetBusinessRules _sepetBusinessRules;

    public CreateSepetCommandHandler(
        ISepetRepository sepetRepository,
        SepetBusinessRules sepetBusinessRules)
    {
        _sepetRepository = sepetRepository;
        _sepetBusinessRules = sepetBusinessRules;
    }

    public async ValueTask<Result<CreatedSepetResponse, DomainError>> HandleAsync(
        CreateSepetCommand request,
        CancellationToken cancellationToken)
    {
        Sepet entity = request.ToEntity();
        await _sepetRepository.AddAsync(entity);
        return Result<CreatedSepetResponse, DomainError>.Success(entity.ToCreatedResponse());
    }
}

#endregion

#region Update Command & Handler

[RequireAuthorization(Admin, Write, SepetlerOperationClaims.Update)]
[Logged]
[Transactional]
public sealed record UpdateSepetCommand(
    int Id,
    string KullaniciId, 
    decimal ToplamTutar, 
    decimal IndirimTutari, 
    bool Kilitli
) : ICommand<Result<UpdatedSepetResponse, DomainError>>;

public class UpdateSepetCommandValidator : AbstractValidator<UpdateSepetCommand>
{
    public UpdateSepetCommandValidator()
    {
        RuleFor(c => c.Id).NotEmpty();
    }
}

internal sealed class UpdateSepetCommandHandler : ICommandHandler<UpdateSepetCommand, Result<UpdatedSepetResponse, DomainError>>
{
    private readonly ISepetRepository _sepetRepository;
    private readonly SepetBusinessRules _sepetBusinessRules;

    public UpdateSepetCommandHandler(
        ISepetRepository sepetRepository,
        SepetBusinessRules sepetBusinessRules)
    {
        _sepetRepository = sepetRepository;
        _sepetBusinessRules = sepetBusinessRules;
    }

    public async ValueTask<Result<UpdatedSepetResponse, DomainError>> HandleAsync(
        UpdateSepetCommand request,
        CancellationToken cancellationToken)
    {
        Sepet? entity = await _sepetRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            cancellationToken: cancellationToken
        );
        await _sepetBusinessRules.SepetShouldExistWhenSelected(entity);

        entity = request.ToEntity();
        await _sepetRepository.UpdateAsync(entity);
        return Result<UpdatedSepetResponse, DomainError>.Success(entity.ToUpdatedResponse());
    }
}

#endregion

#region Delete Command & Handler

[RequireAuthorization(Admin, Write, SepetlerOperationClaims.Delete)]
[Logged]
[Transactional]
public sealed record DeleteSepetCommand(int Id) : ICommand<Result<DeletedSepetResponse, DomainError>>;

public class DeleteSepetCommandValidator : AbstractValidator<DeleteSepetCommand>
{
    public DeleteSepetCommandValidator()
    {
        RuleFor(c => c.Id).NotEmpty();
    }
}

internal sealed class DeleteSepetCommandHandler : ICommandHandler<DeleteSepetCommand, Result<DeletedSepetResponse, DomainError>>
{
    private readonly ISepetRepository _sepetRepository;
    private readonly SepetBusinessRules _sepetBusinessRules;

    public DeleteSepetCommandHandler(
        ISepetRepository sepetRepository,
        SepetBusinessRules sepetBusinessRules)
    {
        _sepetRepository = sepetRepository;
        _sepetBusinessRules = sepetBusinessRules;
    }

    public async ValueTask<Result<DeletedSepetResponse, DomainError>> HandleAsync(
        DeleteSepetCommand request,
        CancellationToken cancellationToken)
    {
        Sepet? entity = await _sepetRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            cancellationToken: cancellationToken
        );
        await _sepetBusinessRules.SepetShouldExistWhenSelected(entity);

        await _sepetRepository.DeleteAsync(entity!);
        return Result<DeletedSepetResponse, DomainError>.Success(entity!.ToDeletedResponse());
    }
}

#endregion
