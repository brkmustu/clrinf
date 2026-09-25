using System.Collections.Generic;
using System.Linq;
using EticaretApp.Domain.Entities;
using EticaretApp.Application.Common.Responses;

namespace EticaretApp.Application.Moduller.Siparisler;

public static class SiparisMappingExtensions
{
    public static Siparis ToEntity(this CreateSiparisCommand command) =>
        new()
        {
            SiparisNo = command.SiparisNo,
            KullaniciId = command.KullaniciId,
            ToplamTutar = command.ToplamTutar,
            Durum = command.Durum,
            KargoTakipNo = command.KargoTakipNo,
        };

    public static CreatedSiparisResponse ToCreatedResponse(this Siparis siparis) =>
        new()
        {
            Id = siparis.Id,
            SiparisNo = siparis.SiparisNo,
            KullaniciId = siparis.KullaniciId,
            ToplamTutar = siparis.ToplamTutar,
            Durum = siparis.Durum,
            KargoTakipNo = siparis.KargoTakipNo,
        };

    public static Siparis ToEntity(this UpdateSiparisCommand command) =>
        new()
        {
            Id = command.Id,
            SiparisNo = command.SiparisNo,
            KullaniciId = command.KullaniciId,
            ToplamTutar = command.ToplamTutar,
            Durum = command.Durum,
            KargoTakipNo = command.KargoTakipNo,
        };

    public static UpdatedSiparisResponse ToUpdatedResponse(this Siparis siparis) =>
        new()
        {
            Id = siparis.Id,
            SiparisNo = siparis.SiparisNo,
            KullaniciId = siparis.KullaniciId,
            ToplamTutar = siparis.ToplamTutar,
            Durum = siparis.Durum,
            KargoTakipNo = siparis.KargoTakipNo,
        };

    public static DeletedSiparisResponse ToDeletedResponse(this Siparis siparis) =>
        new() { Id = siparis.Id };

    public static GetByIdSiparisResponse ToGetByIdResponse(this Siparis siparis) =>
        new()
        {
            Id = siparis.Id,
            SiparisNo = siparis.SiparisNo,
            KullaniciId = siparis.KullaniciId,
            ToplamTutar = siparis.ToplamTutar,
            Durum = siparis.Durum,
            KargoTakipNo = siparis.KargoTakipNo,
        };

    public static GetListSiparisListItemDto ToListItemDto(this Siparis siparis) =>
        new()
        {
            Id = siparis.Id,
            SiparisNo = siparis.SiparisNo,
            KullaniciId = siparis.KullaniciId,
            ToplamTutar = siparis.ToplamTutar,
            Durum = siparis.Durum,
            KargoTakipNo = siparis.KargoTakipNo,
        };

    public static GetListResponse<GetListSiparisListItemDto> ToPaginateResponse(this IList<Siparis> items) =>
        new()
        {
            Items = items.Select(x => x.ToListItemDto()).ToList(),
            Count = items.Count,
            Index = 0,
            Size = items.Count,
            Pages = 1
        };
}
