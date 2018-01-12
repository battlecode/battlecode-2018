import java.util.Random;

import bc.*;

public class Player {

	public static void main(String[] args) {

		GameController gc = new GameController();
		Team myTeam = gc.team();
		Random randomness = new Random(1234);
		Direction[] dirs = Direction.values();
		UnitType[] buildUnits = { UnitType.Knight, UnitType.Ranger };

		int blueprintedFactories = 0;
		int blueprintedRockets = 0;
		int replicatedWorkers = 0;

		while (true) {
			gc.queueResearch(UnitType.Rocket);

			if (gc.round() % 25 == 0) {
				if (gc.planet().equals(Planet.Earth)) {
					System.out.flush();
					System.out.println("jround:" + gc.round());
					System.out
							.println("Number of units: " + gc.myUnits().size());
					System.out.println("Karbonite:" + gc.karbonite());
					System.out.println(
							"Blueprints (Rockets):" + blueprintedRockets);
				} else {
					System.out.println("Waiting for " + gc.unitsInSpace().size()
							+ " adventurers");
					System.out.println(
							"(Already there: " + gc.myUnits().size() + ")");
				}
			}
			try {
				VecUnit units = gc.myUnits();
				for (int i = 0; i < units.size(); i++) {
					Unit unit = units.get(i);

					if (unit.unitType().equals(UnitType.Factory)) {
						UnitType prodUnit = buildUnits[randomness.nextInt(2)];

						VecUnitID garrissonUnits = unit.structureGarrison();
						Direction d = dirs[randomness.nextInt(9)];
						if (garrissonUnits.size() > 0) {

							if (gc.canUnload(unit.id(), d)) {
								gc.unload(unit.id(), d);
							}
						} else if (gc.canProduceRobot(unit.id(), prodUnit)) {
							gc.produceRobot(unit.id(), prodUnit);
						}
						continue;
					} else if (unit.unitType().equals(UnitType.Rocket)) {
						VecUnitID garrissonUnits = unit.structureGarrison();
						if (gc.planet().equals(Planet.Mars)) {
							for (int l = 0; l < 9; l++) {
								if (gc.canUnload(unit.id(),
										Direction.values()[l])) {
									gc.unload(unit.id(), Direction.values()[l]);
									break;
								}
							}
						} else if (garrissonUnits.size() > 6) {
							gc.launchRocket(unit.id(),
									new MapLocation(Planet.Mars,
											randomness.nextInt(20),
											randomness.nextInt(20)));
						} else {
							VecUnit nearby = gc.senseNearbyUnits(
									unit.location().mapLocation(), 2);
							for (int j = 0; j < nearby.size(); j++) {
								if (gc.canLoad(unit.id(), nearby.get(j).id())) {
									gc.load(unit.id(), nearby.get(j).id());
									break;
								}
							}
						}
						continue;
					}

					Location loc = unit.location();
					if (loc.isOnMap()) {
						VecUnit nearby = gc.senseNearbyUnits(loc.mapLocation(),
								Math.max(unit.attackRange(),
										unit.abilityRange()));
						for (int k = 0; k < nearby.size(); k++) {
							Unit other = nearby.get(k);
							if (unit.unitType().equals(UnitType.Worker)
									&& gc.canBuild(unit.id(), other.id())) {
								gc.build(unit.id(), other.id());
								continue;
							}

							if (!other.team().equals(myTeam)
									&& gc.isAttackReady(unit.id())
									&& gc.canAttack(unit.id(), other.id())) {
								gc.attack(unit.id(), other.id());
								continue;
							}
						}

						Direction d = dirs[randomness.nextInt(9)];
						if (unit.unitType().equals(UnitType.Worker)) {
							if (gc.planet().equals(Planet.Earth)) {
								if (blueprintedFactories < 10
										&& gc.karbonite() >= bc
												.bcUnitTypeBlueprintCost(
														UnitType.Factory)
										&& gc.canBlueprint(unit.id(),
												UnitType.Factory, d)) {
									gc.blueprint(unit.id(), UnitType.Factory,
											d);
									blueprintedFactories++;
								} else if (blueprintedRockets < 10
										&& gc.karbonite() >= bc
												.bcUnitTypeBlueprintCost(
														UnitType.Rocket)
										&& gc.canBlueprint(unit.id(),
												UnitType.Rocket, d)) {
									gc.blueprint(unit.id(), UnitType.Rocket, d);
									blueprintedRockets++;

								} else if (gc.canReplicate(unit.id(), d)
										&& replicatedWorkers < 50) {
									gc.replicate(unit.id(), d);

									replicatedWorkers++;
									continue;
								} else if (gc.isMoveReady(unit.id())
										&& gc.canMove(unit.id(), d)) {
									gc.moveRobot(unit.id(), d);

								}
							} else if (gc.isMoveReady(unit.id())
									&& gc.canMove(unit.id(), d)) {
								gc.moveRobot(unit.id(), d);

							}
						} else {
							for (int n = 0; n < 9; n++) {
								if (gc.canHarvest(unit.id(), dirs[n])) {
									gc.harvest(unit.id(), dirs[n]);
									break;
								}
							}
							if (gc.isMoveReady(unit.id())) {
								for (int n = 0; n < 9; n++) {
									if (gc.canMove(unit.id(), dirs[n])) {
										gc.moveRobot(unit.id(), dirs[n]);
										break;
									}
								}
							}
						}
					}
				}
			} catch (Exception e) {

				System.out.println("Oops! Catched:" + e);
			}

			gc.nextTurn();

		}
	}
}
