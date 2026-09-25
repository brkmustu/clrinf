using System.Collections.Generic;
using System.Linq;
using EticaretApp.Domain.Entities;
using EticaretApp.Application.Common.Responses;

namespace EticaretApp.Application.Moduller.Kampanyalar;

public static class KampanyaMappingExtensions
{
    public static Kampanya ToEntity(this CreateKampanyaCommand command) =>
        new()
        {
            Kod = command.Kod,
            Baslik = command.Baslik,
            Deger = command.Deger,
            Aktif = command.Aktif,
        };

    public static CreatedKampanyaResponse ToCreatedResponse(this Kampanya kampanya) =>
        new()
        {
            Id = kampanya.Id,
            Kod = kampanya.Kod,
            Baslik = kampanya.Baslik,
            Deger = kampanya.Deger,
            Aktif = kampanya.Aktif,
        };

    public static Kampanya ToEntity(this UpdateKampanyaCommand command) =>
        new()
        {
            Id = command.Id,
            Kod = command.Kod,
            Baslik = command.Baslik,
            Deger = command.Deger,
            Aktif = command.Aktif,
        };

    public static UpdatedKampanyaResponse ToUpdatedResponse(this Kampanya kampanya) =>
        new()
        {
            Id = kampanya.Id,
            Kod = kampanya.Kod,
            Baslik = kampanya.Baslik,
            Deger = kampanya.Deger,
            Aktif = kampanya.Aktif,
        };

    public static DeletedKampanyaResponse ToDeletedResponse(this Kampanya kampanya) =>
        new() { Id = kampanya.Id };

    public static GetByIdKampanyaResponse ToGetByIdResponse(this Kampanya kampanya) =>
        new()
        {
            Id = kampanya.Id,
            Kod = kampanya.Kod,
            Baslik = kampanya.Baslik,
            Deger = kampanya.Deger,
            Aktif = kampanya.Aktif,
        };

    public static GetListKampanyaListItemDto ToListItemDto(this Kampanya kampanya) =>
        new()
        {
            Id = kampanya.Id,
            Kod = kampanya.Kod,
            Baslik = kampanya.Baslik,
            Deger = kampanya.Deger,
            Aktif = kampanya.Aktif,
        };

    public static GetListResponse<GetListKampanyaListItemDto> ToPaginateResponse(this IList<Kampanya> items) =>
        new()
        {
            Items = items.Select(x => x.ToListItemDto()).ToList(),
            Count = items.Count,
            Index = 0,
            Size = items.Count,
            Pages = 1
        };
}
