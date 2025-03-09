// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN YOUR MODULE SOURCE CODE INSTEAD.

#nullable enable

using System;
using SpacetimeDB.BSATN;
using SpacetimeDB.ClientApi;
using System.Collections.Generic;
using System.Runtime.Serialization;

namespace SpacetimeDB.Types
{
    public sealed partial class RemoteTables
    {
        public sealed class ChannelHandle : RemoteTableHandle<EventContext, Channel>
        {
            protected override string RemoteTableName => "channel";

            public sealed class IdUniqueIndex : UniqueIndexBase<I128>
            {
                protected override I128 GetKey(Channel row) => row.Id;

                public IdUniqueIndex(ChannelHandle table) : base(table) { }
            }

            public readonly IdUniqueIndex Id;

            internal ChannelHandle(DbConnection conn) : base(conn)
            {
                Id = new(this);
            }

            protected override object GetPrimaryKey(Channel row) => row.Id;
        }

        public readonly ChannelHandle Channel;
    }
}
