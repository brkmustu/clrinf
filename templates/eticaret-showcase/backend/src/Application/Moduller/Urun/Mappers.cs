using System.Collections.Generic;
using System.Linq;
using EticaretApp.Domain.Entities;
using EticaretApp.Application.Common.Responses;

namespace EticaretApp.Application.Moduller.Urunlar;

public static class UrunMappingExtensions
{
    public static Urun ToEntity(this CreateUrunCommand command) =>
        new()
        {
            Sku = command.Sku,
            Baslik = command.Baslik,
            Aciklama = command.Aciklama,
            BirimFiyat = command.BirimFiyat,
            ParaBirimi = command.ParaBirimi,
            KategoriYolu = command.KategoriYolu,
            MusaitStok = command.MusaitStok,
            Yayinda = command.Yayinda,
        };

    public static CreatedUrunResponse ToCreatedResponse(this Urun urun) =>
        new()
        {
            Id = urun.Id,
            Sku = urun.Sku,
            Baslik = urun.Baslik,
            Aciklama = urun.Aciklama,
            BirimFiyat = urun.BirimFiyat,
            ParaBirimi = urun.ParaBirimi,
            KategoriYolu = urun.KategoriYolu,
            MusaitStok = urun.MusaitStok,
            Yayinda = urun.Yayinda,
        };

    public static Urun ToEntity(this UpdateUrunCommand command) =>
        new()
        {
            Id = command.Id,
            Sku = command.Sku,
            Baslik = command.Baslik,
            Aciklama = command.Aciklama,
            BirimFiyat = command.BirimFiyat,
            ParaBirimi = command.ParaBirimi,
            KategoriYolu = command.KategoriYolu,
            MusaitStok = command.MusaitStok,
            Yayinda = command.Yayinda,
        };

    public static UpdatedUrunResponse ToUpdatedResponse(this Urun urun) =>
        new()
        {
            Id = urun.Id,
            Sku = urun.Sku,
            Baslik = urun.Baslik,
            Aciklama = urun.Aciklama,
            BirimFiyat = urun.BirimFiyat,
            ParaBirimi = urun.ParaBirimi,
            KategoriYolu = urun.KategoriYolu,
            MusaitStok = urun.MusaitStok,
            Yayinda = urun.Yayinda,
        };

    public static DeletedUrunResponse ToDeletedResponse(this Urun urun) =>
        new() { Id = urun.Id };

    public static GetByIdUrunResponse ToGetByIdResponse(this Urun urun) =>
        new()
        {
            Id = urun.Id,
            Sku = urun.Sku,
            Baslik = urun.Baslik,
            Aciklama = urun.Aciklama,
            BirimFiyat = urun.BirimFiyat,
            ParaBirimi = urun.ParaBirimi,
            KategoriYolu = urun.KategoriYolu,
            MusaitStok = urun.MusaitStok,
            Yayinda = urun.Yayinda,
        };

    public static GetListUrunListItemDto ToListItemDto(this Urun urun) =>
        new()
        {
            Id = urun.Id,
            Sku = urun.Sku,
            Baslik = urun.Baslik,
            Aciklama = urun.Aciklama,
            BirimFiyat = urun.BirimFiyat,
            ParaBirimi = urun.ParaBirimi,
            KategoriYolu = urun.KategoriYolu,
            MusaitStok = urun.MusaitStok,
            Yayinda = urun.Yayinda,
        };

    public static GetListResponse<GetListUrunListItemDto> ToPaginateResponse(this IList<Urun> items) =>
        new()
        {
            Items = items.Select(x => x.ToListItemDto()).ToList(),
            Count = items.Count,
            Index = 0,
            Size = items.Count,
            Pages = 1
        };
}
