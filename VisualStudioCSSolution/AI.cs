using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using BattleCodeCSharp;

namespace BattleCodeCSharp
{
    class AI
    {
        public static void Main()
        {
            Console.WriteLine("Player c# bot starting");
            Direction dir = Direction.North;
            Direction opposite = bc.Direction_opposite(dir);
            Console.WriteLine("Opposite direction of " + dir.ToString() + " is " + opposite.ToString());
            Console.WriteLine("Connection to manager...");
            GameController gc = new GameController();
            Console.WriteLine("Connected!");
            while (true)
            {
                uint round = gc.round();
                Console.WriteLine("Round: " + round);
                VecUnit units = gc.my_units();
                uint len = units.len();
                for (uint i = 0; i < len; i++)
                {
                    Unit unit = units.index(i);
                    ushort id = unit.id();
                    if (gc.can_move(id, Direction.North) > 0 && gc.is_move_ready(id) > 0)
                    {
                        gc.move_robot(id, Direction.North);
                    }
                }
                gc.next_turn();
            }

        }
    }
}