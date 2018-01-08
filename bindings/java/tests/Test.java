import bc.*;

class Test {
    public static void main(String[] args) {
        System.out.println("'"+System.getProperty("java.library.path")+"'");
        //System.loadLibrary("libbattlecode.so");
        System.load("/Users/james/Dev/battlecode-2018/bindings/java/src/bc/libbattlecode.so");
        System.out.println(Direction.North);
        System.out.println(new MapLocation(Planet.Earth, 0, 1));
        System.out.println(new MapLocation(Planet.Earth, 0, 1).equals(new MapLocation(Planet.Earth, 0, 1)));
        System.out.println(new MapLocation(Planet.Earth, 0, 1).getPlanet());
        System.out.println(bc.bcDirectionOpposite(Direction.North));
    }
}