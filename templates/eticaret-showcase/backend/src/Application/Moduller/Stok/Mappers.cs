using System.Collections.Generic;
using System.Linq;
using EticaretApp.Domain.Entities;
using EticaretApp.Application.Common.Responses;

namespace EticaretApp.Application.Moduller.Stoklar;

public static class StokMappingExtensions
{
    public static Stok ToEntity(this CreateStokCommand command) =>
        new()
        {
            Sku = command.Sku,
            Miktar = command.Miktar,
            RezerveMiktar = command.RezerveMiktar,
        };

    public static CreatedStokResponse ToCreatedResponse(this Stok stok) =>
        new()
        {
            Id = stok.Id,
            Sku = stok.Sku,
            Miktar = stok.Miktar,
            RezerveMiktar = stok.RezerveMiktar,
        };

    public static Stok ToEntity(this UpdateStokCommand command) =>
        new()
        {
            Id = command.Id,
            Sku = command.Sku,
            Miktar = command.Miktar,
            RezerveMiktar = command.RezerveMiktar,
        };

    public static UpdatedStokResponse ToUpdatedResponse(this Stok stok) =>
        new()
        {
            Id = stok.Id,
            Sku = stok.Sku,
            Miktar = stok.Miktar,
            RezerveMiktar = stok.RezerveMiktar,
        };

    public static DeletedStokResponse ToDeletedResponse(this Stok stok) =>
        new() { Id = stok.Id };

    public static GetByIdStokResponse ToGetByIdResponse(this Stok stok) =>
        new()
        {
            Id = stok.Id,
            Sku = stok.Sku,
            Miktar = stok.Miktar,
            RezerveMiktar = stok.RezerveMiktar,
        };

    public static GetListStokListItemDto ToListItemDto(this Stok stok) =>
        new()
        {
            Id = stok.Id,
            Sku = stok.Sku,
            Miktar = stok.Miktar,
            RezerveMiktar = stok.RezerveMiktar,
        };

    public static GetListResponse<GetListStokListItemDto> ToPaginateResponse(this IList<Stok> items) =>
        new()
        {
            Items = items.Select(x => x.ToListItemDto()).ToList(),
            Count = items.Count,
            Index = 0,
            Size = items.Count,
            Pages = 1
        };
}
