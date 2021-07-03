using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using Sandbox.Game;
using Sandbox.Game.Entities;
using Sandbox.ModAPI;
using Sandbox.ModAPI.Weapons;
using VRage.Game;
using VRage.Game.Components;
using VRage.Game.ModAPI;
using VRage.ModAPI;
using Sandbox.Common.ObjectBuilders.Definitions;
using Sandbox.Definitions;
using Sandbox.Game.EntityComponents;
using VRage.Game.GUI.TextPanel;

using VRage;
using VRage.Game.Entity;
using VRage.ObjectBuilders;
using VRageMath;
using VRage.Utils;

namespace AdminHealth
{
    [MySessionComponentDescriptor(MyUpdateOrder.BeforeSimulation | MyUpdateOrder.Simulation)]
    public class AdminHealth : MySessionComponentBase
    {
        private int tickTimer;

        public static void PlayerConnectEvent(long playerId)
        {
            if (MyAPIGateway.Players == null || MyAPIGateway.Players.Count == 0)
                return;     //Session Ready-Check
            IMyPlayer player = linkPlayer(playerId);
            if (player == null)
            {
                MyAPIGateway.Utilities.ShowMessage("Player connected: " + playerId, "; could not find IMyPlayer object.");
            }
            else
            {
                MyAPIGateway.Utilities.ShowMessage("Player connected: " + playerId, "; steamId: " + player.SteamUserId + "; character: " + player.Character.EntityId);
            }
        }

        public static IMyPlayer linkPlayer(long playerId, IMyPlayer player = null)
        {
            if (MyAPIGateway.Players == null)
                return null; //Session Ready-Check

            if (player == null)
            {
                List<IMyPlayer> players = new List<IMyPlayer>();
                MyAPIGateway.Players.GetPlayers(players);
                //Get PlayerList

                foreach (IMyPlayer p in players)
                {
                    if (p.IdentityId == playerId)
                    {
                        player = p;
                        break;
                    }
                }
            }

            if (player == null)
                return null;

            if (player.Character == null)
            {
                return player;
            }
            return player;
        }

        public static void PlayerSpawnEvent(long playerId)
        {
            IMyPlayer player = linkPlayer(playerId);
            if (player == null)
            {
                MyAPIGateway.Utilities.ShowMessage("AdminHealth", "Player Spawned, could not find IMyPlayer:" + playerId);
                return;
            }
            else
            {
                MyAPIGateway.Utilities.ShowMessage("AdminHealth", "Player Spawned, found IMyPlayer:" + playerId);
            }

            try
            {

                if (player.PromoteLevel == MyPromoteLevel.Owner || player.PromoteLevel == MyPromoteLevel.Admin)
                {
                    MyVisualScriptLogicProvider.SetPlayersOxygenLevel(playerId, 1.0f);
                    MyVisualScriptLogicProvider.SetPlayersHydrogenLevel(playerId, 1.0f);
                    MyVisualScriptLogicProvider.SetPlayersEnergyLevel(playerId, 1.0f);
                    MyVisualScriptLogicProvider.SetPlayersHealth(playerId, 100f);
                }

                else
                {                }
            }
            catch (Exception e)
            {
                MyAPIGateway.Utilities.ShowMessage("AdminHealth", "Could not set Variables for" + playerId + e);
            }
        }

        public override void Init(MyObjectBuilder_SessionComponent sessionComponent)
        {
            MyAPIGateway.Utilities.ShowMessage("AdminHealth", "Initialized");

            MyVisualScriptLogicProvider.PlayerSpawned += PlayerSpawnEvent;
            MyVisualScriptLogicProvider.PlayerConnected += PlayerConnectEvent;
            MyAPIGateway.Session.OnSessionReady += Session_OnSessionReady;
            //Player death handler
            MyAPIGateway.Session.DamageSystem.RegisterDestroyHandler(0, DestroyHandler);
            base.Init(sessionComponent);
            
            for (var y = 1; y > -2; y--)
            {
                for (var z = -1; z < 2; z++)
                {
                    for (var x = -1; x < 2; x++)
                    {
                        if ((x != 0 && y != 0) || (z != 0 && y != 0) || (x != 0 && z != 0))
                        {
                            points.Add(new Vector3D(x, y, z));
                        }
                    }
                }
            }

            MyLog.Default.WriteLine($"{points.Count}");
            foreach (var pt in points)
            {
                MyLog.Default.WriteLine($"Vector3::new({pt.X}.0, {pt.Y}.0, {pt.Z}.0),");
            }


            subtypes.Add("LargeBlockArmorBlock");
            subtypes.Add("LargeBlockArmorSlope");
            subtypes.Add("LargeBlockArmorCorner");
            subtypes.Add("LargeBlockArmorCornerInv");
            subtypes.Add("LargeBlockArmorCornerSquare");
            subtypes.Add("LargeBlockArmorCornerSquareInverted");
            subtypes.Add("LargeBlockArmorSlope2Base");
            subtypes.Add("LargeBlockArmorSlope2Tip");
            subtypes.Add("LargeHalfArmorBlock");
            subtypes.Add("LargeHalfSlopeArmorBlock");
            subtypes.Add("LargeBlockArmorHalfSlopeCorner");
            subtypes.Add("LargeBlockArmorHalfCorner");
            subtypes.Add("LargeBlockArmorHalfSlopedCorner");
            subtypes.Add("LargeBlockArmorCorner2Base");
            subtypes.Add("LargeBlockArmorCorner2Tip");
            subtypes.Add("LargeBlockArmorInvCorner2Base");
            subtypes.Add("LargeBlockArmorInvCorner2Tip");
            subtypes.Add("LargeBlockArmorHalfSlopeCornerInverted");
            subtypes.Add("LargeBlockArmorHalfSlopeInverted");
            subtypes.Add("LargeBlockArmorSlopedCornerTip");
            subtypes.Add("LargeBlockArmorSlopedCornerBase");
            subtypes.Add("LargeBlockArmorSlopedCorner");
            subtypes.Add("LargeBlockArmorHalfSlopedCornerBase");
            subtypes.Add("AQD_LG_LA_Slab_RaisedCorner_Inset");




            /*
            subtypes.Add("LargeBlockArmorSlope");
            subtypes.Add("LargeHeavyBlockArmorBlock");
            subtypes.Add("LargeBlockArmorCorner");
            subtypes.Add("LargeBlockArmorCornerInv");
            subtypes.Add("LargeBlockArmorCornerSquare");
            */

            /*
            points.Add(new Vector3D(-1.0, 1.0, -1.0));
            points.Add(new Vector3D(1.0, 1.0, -1.0));
            points.Add(new Vector3D(-1.0, 1.0, 1.0));
            points.Add(new Vector3D(1.0, 1.0, 1.0));
            points.Add(new Vector3D(-1.0, -1.0, -1.0));
            points.Add(new Vector3D(1.0, -1.0, -1.0));
            points.Add(new Vector3D(-1.0, -1.0, 1.0));
            points.Add(new Vector3D(1.0, -1.0, 1.0));
            */
        }



        MyCubeGrid currentGrid = null;


        private int hasTicked = 0;

        private List<Vector3D> points = new List<Vector3D>();
        private List<String> subtypes = new List<String>();
        private int currSubtype = 0;

        public bool checkPoint(Vector3D point, MyCubeGrid grid, Vector3D dir)
        {
            var scaledPoint = point * grid.GridSizeHalf;
            var target = Vector3D.Transform(grid.PositionComp.LocalAABB.Center + scaledPoint, grid.PositionComp.WorldMatrixRef) - (dir * 0.1);
            var source = target + (dir * 5.0);

            //MyLog.Default.WriteLine($"src={source} trg={target}");

            List<IHitInfo> hits = new List<IHitInfo>();
            for (var x = -3; x < 4; x++)
            {
                for (var y = -3; y < 4; y++)
                {
                    for (var z = -3; z < 4; z++)
                    {
                        var p = new Vector3D(x, y, z) * 0.0333;
                        MyAPIGateway.Physics.CastRay(source + p, target + p, hits);
                        if (hits.Count > 0)
                        {
                            return true;
                        }
                    }
                }
            }

            return false;
        }

        public override void UpdateBeforeSimulation()
        {
            base.UpdateBeforeSimulation();
            tickTimer++;


            if (hasTicked > 5)
            {
                if (currSubtype < subtypes.Count)
                {
                    var type = subtypes[currSubtype];
                    currSubtype++;
                    var ent = SpawnBlock(type, "TestGrid", true, true, true, true, true, 0);
                    currentGrid = ent as MyCubeGrid;
                    if (currentGrid == null)
                    {
                        MyLog.Default.WriteLine("Failed to cast entity to cubegrid!");
                    }
                    var res = new List<bool>();
                    var i = 0;
                    foreach (var point in points)
                    {
                        Vector3D dir;
                        var hit = false;
                        if (point.Z < -0.9)
                        {
                            //MyLog.Default.WriteLine($"i={i} FWD");
                            hit = hit || checkPoint(point, currentGrid, currentGrid.PositionComp.WorldMatrixRef.Forward);
                        }
                        if (point.Z > 0.9)
                        {
                            //MyLog.Default.WriteLine($"i={i} BACK");
                            hit = hit || checkPoint(point, currentGrid, currentGrid.PositionComp.WorldMatrixRef.Backward);
                        }
                        if (point.X < -0.9)
                        {
                            //MyLog.Default.WriteLine($"i={i} LEFT");
                            hit = hit || checkPoint(point, currentGrid, currentGrid.PositionComp.WorldMatrixRef.Left);
                        }
                        if (point.X > 0.9)
                        {
                            //MyLog.Default.WriteLine($"i={i} RIGHT");
                            hit = hit || checkPoint(point, currentGrid, currentGrid.PositionComp.WorldMatrixRef.Right);
                        }
                        if (point.Y < -0.9)
                        {
                            //MyLog.Default.WriteLine($"i={i} DOWN");
                            hit = hit || checkPoint(point, currentGrid, currentGrid.PositionComp.WorldMatrixRef.Down);
                        }
                        if (point.Y > 0.9)
                        {
                            //MyLog.Default.WriteLine($"i={i} UP");
                            hit = hit || checkPoint(point, currentGrid, currentGrid.PositionComp.WorldMatrixRef.Up);
                        }
                        res.Add(hit);
                        i++;
                    }

                    var def = MyDefinitionManager.Static.GetCubeBlockDefinition(MyDefinitionId.Parse($"MyObjectBuilder_CubeBlock/{type}"));
                    var pair = MyDefinitionManager.Static.GetDefinitionGroup(def.BlockPairName);
                    MyLog.Default.WriteLine($"CubeBlock::new([{String.Join(",", res)}], \"{pair.Large.Id.SubtypeName}\", \"{pair.Small.Id.SubtypeName}\"),");
                    currentGrid.Close();
                    currentGrid = null;
                }
            }


            if (tickTimer <= 900)
            {
                hasTicked++;
                return;
            }

            if (tickTimer >= 901)
            {
                tickTimer = 0;
            }

            var adminList = new List<IMyPlayer>();
            MyAPIGateway.Players.GetPlayers(adminList);

            if (adminList.Count == 0)
            {
                return;
            }

            foreach (IMyPlayer admin in adminList)
            {
                if (admin.IsBot == true || admin.Character == null)
                {
                    continue;
                }

                if (admin == null)
                {
                    return;
                }

                if (admin.PromoteLevel == MyPromoteLevel.Owner || admin.PromoteLevel == MyPromoteLevel.Admin)
                {
                    var health = MyVisualScriptLogicProvider.GetPlayersHealth(admin.IdentityId);
                    var energy = MyVisualScriptLogicProvider.GetPlayersEnergyLevel(admin.IdentityId);
                    var oxygen = MyVisualScriptLogicProvider.GetPlayersOxygenLevel(admin.IdentityId);
                    var hydrogen = MyVisualScriptLogicProvider.GetPlayersHydrogenLevel(admin.IdentityId);

                    if (health > 75 && energy > 0.75f && oxygen > 0.75f && hydrogen > 0.75f)
                    {
                        continue;
                    }

                    if (health <= 0)
                    {
                        continue;
                    }

                    if (health <= 50)
                    {
                        MyVisualScriptLogicProvider.SetPlayersHealth(admin.IdentityId, 100.0f);
                        MyVisualScriptLogicProvider.ShowNotification("Out of health? You must have forgot invulnerablity!", 3000, "White", admin.IdentityId);
                    }
                    if (energy <= 0.75f)
                    {
                        MyVisualScriptLogicProvider.SetPlayersEnergyLevel(admin.IdentityId, 1.0f);
                        MyVisualScriptLogicProvider.ShowNotification("Look at you, expending your energy doin tickets and shit!", 3000, "White", admin.IdentityId);
                    }
                    if (oxygen <= 0.75f)
                    {
                        MyVisualScriptLogicProvider.SetPlayersOxygenLevel(admin.IdentityId, 1.0f);
                        MyVisualScriptLogicProvider.ShowNotification("Breathe in to the count of 4, and out to the count of 8...", 3000, "White", admin.IdentityId);
                    }
                    if (hydrogen <= 0.75f)
                    {
                        MyVisualScriptLogicProvider.SetPlayersHydrogenLevel(admin.IdentityId, 1.0f);
                        MyVisualScriptLogicProvider.ShowNotification("Oh look at the pelican... fly little pelican", 3000, "White", admin.IdentityId);
                    }
                }
            }
        }

        private void Session_OnSessionReady()
        {
            //Link all connected players now.
            List<IMyPlayer> players = new List<IMyPlayer>();
            MyAPIGateway.Players.GetPlayers(players);
            foreach (IMyPlayer player in players)
            {
                linkPlayer(-1, player);
            }

        }

        private static void DestroyHandler(object target, MyDamageInformation info)
        {
            try
            {
                if (!MyAPIGateway.Multiplayer.IsServer)
                {
                    return;
                }

                if (!(target is IMyCharacter))
                {
                    return;
                }

                var character = (IMyEntity)target;
                var player = MyAPIGateway.Players.GetPlayerControllingEntity(character);
                var playerName = player.DisplayName;

            }
            catch (Exception e)
            {
                MyAPIGateway.Utilities.ShowMessage("AdminHealth", "DestroyHandler Exception: " + e);
            }
        }





        private static readonly MyObjectBuilder_CubeGrid CubeGridBuilder = new MyObjectBuilder_CubeGrid()
        {
            EntityId = 0,
            GridSizeEnum = MyCubeSize.Large,
            IsStatic = true,
            Skeleton = new List<BoneInfo>(),
            LinearVelocity = Vector3.Zero,
            AngularVelocity = Vector3.Zero,
            ConveyorLines = new List<MyObjectBuilder_ConveyorLine>(),
            BlockGroups = new List<MyObjectBuilder_BlockGroup>(),
            Handbrake = false,
            XMirroxPlane = null,
            YMirroxPlane = null,
            ZMirroxPlane = null,
            PersistentFlags = MyPersistentEntityFlags2.InScene,
            Name = "ArtificialCubeGrid",
            DisplayName = "FieldEffect",
            CreatePhysics = false,
            DestructibleBlocks = true,
            PositionAndOrientation = new MyPositionAndOrientation(Vector3D.Zero, Vector3D.Forward, Vector3D.Up),

            CubeBlocks = new List<MyObjectBuilder_CubeBlock>()
                {
                    new MyObjectBuilder_CubeBlock()
                    {
                        EntityId = 0,
                        BlockOrientation = EntityOrientation,
                        SubtypeName = string.Empty,
                        Name = string.Empty,
                        Min = Vector3I.Zero,
                        Owner = 0,
                        ShareMode = MyOwnershipShareModeEnum.None,
                        DeformationRatio = 0,
                    }
                }
        };

        private static readonly SerializableBlockOrientation EntityOrientation = new SerializableBlockOrientation(Base6Directions.Direction.Forward, Base6Directions.Direction.Up);


        public static MyEntity SpawnBlock(string subtypeId, string name, bool isVisible = false, bool hasPhysics = false, bool isStatic = false, bool toSave = false, bool destructible = false, long ownerId = 0)
        {
            try
            {
                CubeGridBuilder.Name = name;
                CubeGridBuilder.CubeBlocks[0].SubtypeName = subtypeId;
                CubeGridBuilder.CreatePhysics = hasPhysics;
                CubeGridBuilder.IsStatic = isStatic;
                CubeGridBuilder.DestructibleBlocks = destructible;
                var ent = (MyEntity)MyAPIGateway.Entities.CreateFromObjectBuilder(CubeGridBuilder);

                ent.Flags &= ~EntityFlags.Save;
                ent.Render.Visible = isVisible;
                MyAPIGateway.Entities.AddEntity(ent);

                return ent;
            }
            catch (Exception ex)
            {

                MyLog.Default.WriteLine("Exception in Spawn");
                MyLog.Default.WriteLine($"{ex}");
                return null;
            }
        }
    }
}
