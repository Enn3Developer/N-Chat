// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN YOUR MODULE SOURCE CODE INSTEAD.

#nullable enable

using System;
using System.Collections.Generic;
using System.Runtime.Serialization;

namespace SpacetimeDB.Types
{
    [SpacetimeDB.Type]
    [DataContract]
    public sealed partial class Message
    {
        [DataMember(Name = "id")]
        public I128 Id;
        [DataMember(Name = "sender")]
        public string Sender;
        [DataMember(Name = "channel_id")]
        public I128 ChannelId;
        [DataMember(Name = "sent")]
        public SpacetimeDB.Timestamp Sent;
        [DataMember(Name = "text")]
        public string Text;

        public Message(
            I128 Id,
            string Sender,
            I128 ChannelId,
            SpacetimeDB.Timestamp Sent,
            string Text
        )
        {
            this.Id = Id;
            this.Sender = Sender;
            this.ChannelId = ChannelId;
            this.Sent = Sent;
            this.Text = Text;
        }

        public Message()
        {
            this.Sender = "";
            this.Text = "";
        }
    }
}
