using System.Threading;
using System.Threading.Tasks;
using EticaretApp.Application.Common.Rules;
using EticaretApp.Application.Common.Exceptions;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Application.Moduller.Siparisler;

public static class SiparisBusinessMessages
{
    public const string SectionName = "Siparis";
    public const string SiparisNotExists = "Siparis bulunamadı.";
    public const string SiparisAlreadyExists = "Bu Siparis zaten mevcut.";
}

public class SiparisBusinessRules : BaseBusinessRules
{
    private readonly ISiparisRepository _siparisRepository;

    public SiparisBusinessRules(ISiparisRepository siparisRepository)
    {
        _siparisRepository = siparisRepository;
    }

    public async Task SiparisShouldExistWhenSelected(Siparis? siparis)
    {
        if (siparis == null)
            throw new BusinessException(SiparisBusinessMessages.SiparisNotExists);
    }

    public async Task SiparisIdShouldExistWhenSelected(int id, CancellationToken cancellationToken)
    {
        Siparis? siparis = await _siparisRepository.GetAsync(
            predicate: x => x.Id.Equals(id),
            enableTracking: false,
            cancellationToken: cancellationToken
        );
        await SiparisShouldExistWhenSelected(siparis);
    }
}
