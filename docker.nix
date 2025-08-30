{
  dockerTools,
  buildEnv,
  website,
}:
dockerTools.buildImage {
  name = "website";
  tag = "latest";

  copyToRoot = buildEnv {
    name = "image-root";
    paths = [ website ];
    pathsToLink = [ "/bin" ];
  };

  config = {
    Cmd = [ "/bin/website" ];
    WorkingDir = "/";
  };
}
