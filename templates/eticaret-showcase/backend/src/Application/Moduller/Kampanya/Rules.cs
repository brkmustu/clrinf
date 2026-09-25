using System.Threading;
using System.Threading.Tasks;
using EticaretApp.Application.Common.Rules;
using EticaretApp.Application.Common.Exceptions;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Application.Moduller.Kampanyalar;

public static class KampanyaBusinessMessages
{
    public const string SectionName = "Kampanya";
    public const string KampanyaNotExists = "Kampanya bulunamadı.";
    public const string KampanyaAlreadyExists = "Bu Kampanya zaten mevcut.";
}

public class KampanyaBusinessRules : BaseBusinessRules
{
    private readonly IKampanyaRepository _kampanyaRepository;

    public KampanyaBusinessRules(IKampanyaRepository kampanyaRepository)
    {
        _kampanyaRepository = kampanyaRepository;
    }

    public async Task KampanyaShouldExistWhenSelected(Kampanya? kampanya)
    {
        if (kampanya == null)
            throw new BusinessException(KampanyaBusinessMessages.KampanyaNotExists);
    }

    public async Task KampanyaIdShouldExistWhenSelected(int id, CancellationToken cancellationToken)
    {
        Kampanya? kampanya = await _kampanyaRepository.GetAsync(
            predicate: x => x.Id.Equals(id),
            enableTracking: false,
            cancellationToken: cancellationToken
        );
        await KampanyaShouldExistWhenSelected(kampanya);
    }
}
