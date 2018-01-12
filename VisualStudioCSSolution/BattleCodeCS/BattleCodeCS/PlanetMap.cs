//------------------------------------------------------------------------------
// <auto-generated />
//
// This file was automatically generated by SWIG (http://www.swig.org).
// Version 3.0.8
//
// Do not make changes to this file unless you know what you are doing--modify
// the SWIG interface file instead.
//------------------------------------------------------------------------------


public class PlanetMap : global::System.IDisposable {
  private global::System.Runtime.InteropServices.HandleRef swigCPtr;
  protected bool swigCMemOwn;

  internal PlanetMap(global::System.IntPtr cPtr, bool cMemoryOwn) {
    swigCMemOwn = cMemoryOwn;
    swigCPtr = new global::System.Runtime.InteropServices.HandleRef(this, cPtr);
  }

  internal static global::System.Runtime.InteropServices.HandleRef getCPtr(PlanetMap obj) {
    return (obj == null) ? new global::System.Runtime.InteropServices.HandleRef(null, global::System.IntPtr.Zero) : obj.swigCPtr;
  }

  ~PlanetMap() {
    Dispose();
  }

  public virtual void Dispose() {
    lock(this) {
      if (swigCPtr.Handle != global::System.IntPtr.Zero) {
        if (swigCMemOwn) {
          swigCMemOwn = false;
          bcPINVOKE.delete_PlanetMap(swigCPtr);
        }
        swigCPtr = new global::System.Runtime.InteropServices.HandleRef(null, global::System.IntPtr.Zero);
      }
      global::System.GC.SuppressFinalize(this);
    }
  }

  public PlanetMap() : this(bcPINVOKE.new_PlanetMap(), true) {
    if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
  }

  public void validate() {
    bcPINVOKE.PlanetMap_validate(swigCPtr);
    if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
  }

  public byte on_map(MapLocation location) {
    byte ret = bcPINVOKE.PlanetMap_on_map(swigCPtr, MapLocation.getCPtr(location));
    if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
    return ret;
  }

  public byte is_passable_terrain_at(MapLocation location) {
    byte ret = bcPINVOKE.PlanetMap_is_passable_terrain_at(swigCPtr, MapLocation.getCPtr(location));
    if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
    return ret;
  }

  public uint initial_karbonite_at(MapLocation location) {
    uint ret = bcPINVOKE.PlanetMap_initial_karbonite_at(swigCPtr, MapLocation.getCPtr(location));
    if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
    return ret;
  }

  public PlanetMap clone() {
    global::System.IntPtr cPtr = bcPINVOKE.PlanetMap_clone(swigCPtr);
    PlanetMap ret = (cPtr == global::System.IntPtr.Zero) ? null : new PlanetMap(cPtr, false);
    if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
    return ret;
  }

  public string to_json() {
    string ret = bcPINVOKE.PlanetMap_to_json(swigCPtr);
    if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
    return ret;
  }

  public Planet planet {
    set {
      bcPINVOKE.PlanetMap_planet_set(swigCPtr, (int)value);
      if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
    } 
    get {
      Planet ret = (Planet)bcPINVOKE.PlanetMap_planet_get(swigCPtr);
      if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
      return ret;
    } 
  }

  public uint height {
    set {
      bcPINVOKE.PlanetMap_height_set(swigCPtr, value);
      if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
    } 
    get {
      uint ret = bcPINVOKE.PlanetMap_height_get(swigCPtr);
      if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
      return ret;
    } 
  }

  public uint width {
    set {
      bcPINVOKE.PlanetMap_width_set(swigCPtr, value);
      if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
    } 
    get {
      uint ret = bcPINVOKE.PlanetMap_width_get(swigCPtr);
      if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
      return ret;
    } 
  }

  public VecUnit initial_units {
    set {
      bcPINVOKE.PlanetMap_initial_units_set(swigCPtr, VecUnit.getCPtr(value));
      if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
    } 
    get {
      global::System.IntPtr cPtr = bcPINVOKE.PlanetMap_initial_units_get(swigCPtr);
      VecUnit ret = (cPtr == global::System.IntPtr.Zero) ? null : new VecUnit(cPtr, false);
      if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
      return ret;
    } 
  }

}
