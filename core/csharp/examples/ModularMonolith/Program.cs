using ClrinfCS.Adapters.Sqlite;
using ClrinfCS.Core;
using ClrinfCS.Examples.Documents;

var context = new RequestContext("tenant-demo", Guid.NewGuid().ToString(), Guid.NewGuid().ToString());
var store = new SqliteStore(args.ElementAtOrDefault(0) ?? "monolith.db");
var dispatcher = new Dispatcher.Builder().Register(new ApproveDocumentHandler(store)).Build();
await dispatcher.SendAsync(new ApproveDocument("document-example"), context);
foreach (var delivery in store.Claim(context, 100, TimeSpan.FromMinutes(1)))
{
    DocumentProjection.Apply(store, context, delivery.Event);
    if (!store.Complete(context, delivery)) throw new InvalidOperationException("The outbox lease expired.");
}
Console.WriteLine(store.Execute(context, tx => tx.Get("projection/document-example")));
