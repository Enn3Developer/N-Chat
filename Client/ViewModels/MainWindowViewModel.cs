using System.Collections.ObjectModel;
using DynamicData;
using ReactiveUI;
using SpacetimeDB.Types;

namespace Client.ViewModels;

public class MainWindowViewModel : ViewModelBase
{
    public static MainWindowViewModel Instance { get; private set; } = null!;

    public ViewModelBase ContentViewModel
    {
        get;
        private set => this.RaiseAndSetIfChanged(ref field, value);
    }

    public ObservableCollection<ChannelViewModel> Channels { get; private set; } = [];

    public SpacetimeDB? SpacetimeDb { get; private set; }

    public MainWindowViewModel()
    {
        Instance = this;
        ContentViewModel = new MainViewModel();
    }

    public void InitSpacetimeDb()
    {
        SpacetimeDb ??= new SpacetimeDB(Callback, TickCallback, (connection, identity) =>
        {
            connection.SubscriptionBuilder().OnApplied(context =>
            {
                Channels.AddRange(context.Db.Channel.Iter()
                    .Join(context.Db.Member.Iter(), channel => channel.Id,
                        member => member.ChannelId,
                        (channel, member) => member.UserId == identity ? channel : null)
                    .Where(channel => channel != null)
                    .Select(channel => new ChannelViewModel { Name = channel!.Name, Id = channel.Id }));
                foreach (var channelViewModel in Channels)
                {
                    Console.WriteLine(channelViewModel.Name);
                }
            }).Subscribe([
                "SELECT * FROM channel",
                "SELECT * FROM member",
            ]);
        });
    }

    private void Callback(DbConnection connection)
    {
    }

    private void TickCallback(DbConnection connection)
    {
    }

    public override void OnClose()
    {
        ContentViewModel.OnClose();
        SpacetimeDb?.Stop();
    }
}