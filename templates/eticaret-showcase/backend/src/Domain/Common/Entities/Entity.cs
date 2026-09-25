using System;

namespace EticaretApp.Domain.Common.Entities;

public abstract class Entity<TId> : IEntity<TId>
{
    public TId Id { get; set; } = default!;
    public DateTime CreatedDate { get; set; } = DateTime.UtcNow;
    public DateTime? UpdatedDate { get; set; }
    public DateTime? DeletedDate { get; set; }

    protected Entity() { }

    protected Entity(TId id)
    {
        Id = id;
    }
}
