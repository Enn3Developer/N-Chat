using System.Collections.ObjectModel;
using DynamicData;
using ReactiveUI;
using SpacetimeDB;
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
        SpacetimeDb ??= new SpacetimeDB(Callback, TickCallback, connection =>
        {
            connection.SubscriptionBuilder().OnApplied(_ => { }).Subscribe([
                "SELECT * FROM channel",
                "SELECT * FROM member",
            ]);
        });
    }

    /// <summary>
    /// Callback for registering callbacks from the <see cref="DbConnection"/>
    /// </summary>
    /// <param name="connection">the db where to register callbacks</param>
    private void Callback(DbConnection connection)
    {
        connection.Db.Member.OnInsert += (context, row) =>
        {
            if (row.UserId == context.Identity!)
            {
                var channel = context.Db.Channel.Iter().First(channel => row.ChannelId == channel.Id);
                Channels.Add(new ChannelViewModel { Name = channel.Name, Id = channel.Id });
            }
        };
        connection.Reducers.OnSetName += (ctx, name) =>
        {
            if (ctx.Event.Status is Status.Failed failed)
            {
                Console.Error.WriteLine($"Failed to set name: {failed.Failed_}");
            }
        };
    }

    /// <summary>
    /// Callback where to use reducers from the <see cref="DbConnection"/>
    /// </summary>
    /// <param name="connection">the db where to use reducers</param>
    private void TickCallback(DbConnection connection)
    {
    }

    public override void OnClose()
    {
        ContentViewModel.OnClose();
        SpacetimeDb?.Stop();
    }
}